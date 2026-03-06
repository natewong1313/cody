use crate::backend::{
    db::{Database, DatabaseStartupError},
    harness::{Harness, opencode::OpencodeHarness},
    repo::{message::MessageRepo, project::ProjectRepo, session::SessionRepo},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{sync::watch, task::JoinHandle};
use tonic::transport::Server;
use uuid::Uuid;

pub use models::project_model::ProjectModel;
pub use models::session_model::SessionModel;
mod agent;
mod db;
mod harness;
mod models;
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
    tonic::include_proto!("messages");
}
use proto_message::messages_server::MessagesServer;
pub use proto_message::{
    CreateUserMessageReply, CreateUserMessageRequest, ListMessagesBySessionReply,
    ListMessagesBySessionRequest, SubscribeMessagesBySessionReply,
    SubscribeMessagesBySessionRequest, messages_client::MessagesClient,
};

pub struct BackendContext {
    db: Arc<Database>,
    harness: OpencodeHarness,
}

impl Clone for BackendContext {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            harness: self.harness.clone(),
        }
    }
}

impl BackendContext {
    fn new(db: Database, harness: OpencodeHarness) -> Self {
        Self {
            db: Arc::new(db),
            harness,
        }
    }
}

pub struct BackendService {
    ctx: BackendContext,
    project_repo: ProjectRepo,
    projects_sender: watch::Sender<Vec<ProjectModel>>,
    project_sender_by_id: Mutex<HashMap<Uuid, watch::Sender<Option<ProjectModel>>>>,
    session_repo: SessionRepo,
    message_repo: MessageRepo,
}

#[derive(thiserror::Error, Debug)]
pub enum BackendServiceError {
    #[error("Database initialization failed: {0}")]
    Database(#[from] DatabaseStartupError),
    #[error("Harness initialization failed: {0}")]
    Harness(String),
}

impl BackendService {
    pub async fn new() -> Result<Self, BackendServiceError> {
        let db = Database::new().await?;
        let harness =
            OpencodeHarness::new().map_err(|e| BackendServiceError::Harness(e.to_string()))?;
        let ctx = BackendContext::new(db, harness);

        let project_repo = ProjectRepo::new(ctx.clone());
        let (projects_sender, _) = watch::channel(Vec::new());
        let project_sender_by_id = Mutex::new(HashMap::new());
        let session_repo = SessionRepo::new(ctx.clone());
        let message_repo = MessageRepo::new(ctx.clone());

        Ok(Self {
            ctx,
            project_repo,
            projects_sender,
            project_sender_by_id,
            session_repo,
            message_repo,
        })
    }
}

pub async fn spawn_backend(
    addr: SocketAddr,
) -> Result<JoinHandle<Result<(), tonic::transport::Error>>, BackendServiceError> {
    let backend = Arc::new(BackendService::new().await?);

    let project_service = ProjectServer::new(backend.clone());
    let session_service = SessionServer::new(backend.clone());
    let message_service = MessagesServer::new(backend.clone());

    Ok(tokio::spawn(async move {
        log::info!("gRPC backend listening on {addr}");
        Server::builder()
            .add_service(project_service)
            .add_service(session_service)
            .add_service(message_service)
            .serve(addr)
            .await
    }))
}
