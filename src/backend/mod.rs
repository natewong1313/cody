use crate::backend::{
    db::{Database, DatabaseStartupError, sqlite::Sqlite},
    harness::{Harness, opencode::OpencodeHarness},
    repo::{message::MessageRepo, project::ProjectRepo, session::SessionRepo},
};
use futures::StreamExt;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{sync::watch, task::JoinHandle};
use tonic::transport::Server;
use uuid::Uuid;

pub use repo::project::Project;
pub use repo::session::Session;
mod db;
mod harness;
pub mod proto_utils;
mod repo;
mod service;

pub(crate) mod proto_project {
    tonic::include_proto!("project");
}
use proto_project::project_server::ProjectServer;
pub use proto_project::{
    SubscribeProjectReply, SubscribeProjectRequest, SubscribeProjectsReply,
    SubscribeProjectsRequest, project_client::ProjectClient,
};

pub(crate) mod proto_session {
    tonic::include_proto!("session");
}
use proto_session::session_server::SessionServer;
pub use proto_session::{
    ListSessionsByProjectReply, ListSessionsByProjectRequest, session_client::SessionClient,
};

pub(crate) mod proto_message {
    tonic::include_proto!("message");
}
use proto_message::message_server::MessageServer;

pub struct BackendContext<D>
where
    D: Database,
{
    db: Arc<D>,
    harness: OpencodeHarness,
}

impl<D> Clone for BackendContext<D>
where
    D: Database,
{
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            harness: self.harness.clone(),
        }
    }
}

impl<D> BackendContext<D>
where
    D: Database,
{
    fn new(db: D, harness: OpencodeHarness) -> Self {
        Self {
            db: Arc::new(db),
            harness,
        }
    }
}

pub struct BackendService {
    project_repo: ProjectRepo<Sqlite>,
    projects_sender: watch::Sender<Vec<Project>>,
    project_sender_by_id: Mutex<HashMap<Uuid, watch::Sender<Option<Project>>>>,
    session_repo: SessionRepo<Sqlite>,
    message_repo: MessageRepo<Sqlite>,
    message_sender_by_session_id:
        Mutex<HashMap<Uuid, watch::Sender<Vec<proto_message::MessageModel>>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum BackendServiceError {
    #[error("Database initialization failed: {0}")]
    Database(#[from] DatabaseStartupError),
    #[error("Harness initialization failed: {0}")]
    Harness(String),
}

impl BackendService {
    pub fn new() -> Result<Self, BackendServiceError> {
        let db = Sqlite::new()?;
        let harness =
            OpencodeHarness::new().map_err(|e| BackendServiceError::Harness(e.to_string()))?;
        let ctx = BackendContext::new(db, harness);

        let project_repo = ProjectRepo::new(ctx.clone());
        let (projects_sender, _) = watch::channel(Vec::new());
        let project_sender_by_id = Mutex::new(HashMap::new());
        let session_repo = SessionRepo::new(ctx.clone());
        let message_repo = MessageRepo::new(ctx.clone());

        Ok(Self {
            project_repo,
            projects_sender,
            project_sender_by_id,
            session_repo,
            message_repo,
            message_sender_by_session_id: Mutex::new(HashMap::new()),
        })
    }

    async fn publish_session_messages(&self, session_id: Uuid) {
        let sender = match self.message_sender_by_session_id.lock() {
            Ok(mut map) => map
                .entry(session_id)
                .or_insert_with(|| watch::channel(Vec::new()).0)
                .clone(),
            Err(_) => {
                log::error!("message sender map lock poisoned");
                return;
            }
        };

        let messages = match self.message_repo.list_messages(&session_id, None).await {
            Ok(messages) => messages,
            Err(err) => {
                log::error!("failed listing messages for {}: {}", session_id, err);
                return;
            }
        };

        let payload: Vec<proto_message::MessageModel> =
            messages.into_iter().map(Into::into).collect();
        sender.send_replace(payload);
    }

    async fn reconcile_subscribed_sessions(&self) {
        let session_ids: Vec<Uuid> = match self.message_sender_by_session_id.lock() {
            Ok(map) => map.keys().copied().collect(),
            Err(_) => {
                log::error!("message sender map lock poisoned");
                return;
            }
        };

        for session_id in session_ids {
            match self
                .message_repo
                .reconcile_session_messages(&session_id, None)
                .await
            {
                Ok(()) => self.publish_session_messages(session_id).await,
                Err(err) => {
                    log::warn!(
                        "failed reconciling subscribed session {}: {}",
                        session_id,
                        err
                    );
                }
            }
        }
    }
}

fn spawn_message_ingestor(backend: Arc<BackendService>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let stream_result = backend.message_repo.get_event_stream().await;
            let mut stream = match stream_result {
                Ok(stream) => stream,
                Err(err) => {
                    log::warn!("message ingestor stream connect failed: {}", err);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            backend.reconcile_subscribed_sessions().await;

            while let Some(event_result) = stream.next().await {
                let event = match event_result {
                    Ok(event) => event,
                    Err(err) => {
                        log::warn!("message ingestor stream error: {}", err);
                        break;
                    }
                };

                let parsed = serde_json::from_str::<harness::OpencodeGlobalEvent>(&event.data);
                let parsed = match parsed {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        let payload_snippet: String = event.data.chars().take(200).collect();
                        log::warn!(
                            "ignoring unparseable opencode event: {}; payload={}",
                            err,
                            payload_snippet
                        );
                        continue;
                    }
                };

                match backend.message_repo.ingest_event(parsed).await {
                    Ok(Some(session_id)) => backend.publish_session_messages(session_id).await,
                    Ok(None) => {}
                    Err(err) => {
                        log::error!("failed ingesting opencode event: {}", err);
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    })
}

pub fn spawn_backend(
    addr: SocketAddr,
) -> Result<JoinHandle<Result<(), tonic::transport::Error>>, BackendServiceError> {
    let backend = Arc::new(BackendService::new()?);

    let project_service = ProjectServer::new(backend.clone());
    let session_service = SessionServer::new(backend.clone());
    let message_service = MessageServer::new(backend.clone());
    let ingestor_handle = spawn_message_ingestor(backend);

    Ok(tokio::spawn(async move {
        log::info!("gRPC backend listening on {addr}");
        let result = Server::builder()
            .add_service(project_service)
            .add_service(session_service)
            .add_service(message_service)
            .serve(addr)
            .await;

        ingestor_handle.abort();
        let _ = ingestor_handle.await;
        result
    }))
}
