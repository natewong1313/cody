use crate::backend::{
    db::{Database, DatabaseStartupError, sqlite::Sqlite},
    harness::{Harness, opencode::OpencodeHarness},
    repo::{project::ProjectRepo, session::SessionRepo},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{sync::watch, task::JoinHandle};
use tonic::transport::Server;

pub use repo::project::Project;
pub use repo::session::Session;
mod db;
mod harness;
pub mod proto_utils;
mod repo;
mod service;
mod state;

pub mod proto_project {
    tonic::include_proto!("project");
}
use proto_project::project_server::ProjectServer;

pub mod proto_session {
    tonic::include_proto!("session");
}
use proto_session::session_server::SessionServer;

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
    session_repo: SessionRepo<Sqlite>,
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
        let session_repo = SessionRepo::new(ctx.clone());

        Ok(Self {
            project_repo,
            projects_sender,
            session_repo,
        })
    }
}

pub fn spawn_backend(
    addr: SocketAddr,
) -> Result<JoinHandle<Result<(), tonic::transport::Error>>, BackendServiceError> {
    let backend = Arc::new(BackendService::new()?);

    let project_service = ProjectServer::new(backend.clone());
    let session_service = SessionServer::new(backend);

    Ok(tokio::spawn(async move {
        log::info!("gRPC backend listening on {addr}");
        Server::builder()
            .add_service(project_service)
            .add_service(session_service)
            .serve(addr)
            .await
    }))
}
