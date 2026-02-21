use crate::backend::{
    data::{project::ProjectRepoError, session::SessionRepoError},
    harness::{Harness, opencode::OpencodeHarness},
    state::StateError,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use uuid::Uuid;

pub use data::project::Project;
pub use data::session::Session;

mod data;
mod db;
mod harness;
mod local;
mod state;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("project not found: {0}")]
    ProjectNotFound(Uuid),
    #[error("not found")]
    NotFound,
    #[error("backend unavailable: {0}")]
    Unavailable(String),
}

impl From<ProjectRepoError> for BackendError {
    fn from(err: ProjectRepoError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<SessionRepoError> for BackendError {
    fn from(err: SessionRepoError) -> Self {
        match err {
            SessionRepoError::ProjectNotFound(id) => Self::ProjectNotFound(id),
            SessionRepoError::Harness(message) => Self::Unavailable(message),
            SessionRepoError::Database(e) => Self::Internal(e.to_string()),
        }
    }
}

impl From<StateError> for BackendError {
    fn from(err: StateError) -> Self {
        Self::Internal(err.to_string())
    }
}

// #[derive(Clone)]
// pub enum BackendEvent {
//     ProjectsLoaded(Vec<Project>),
// }

// // We'll need to make this a trait at some point when we add wasm
// #[derive(Clone)]
// pub struct BackendEventSender {
//     sender: broadcast::Sender<BackendEvent>,
// }
//
// impl BackendEventSender {
//     fn new() -> Self {
//         let (sender, _) = broadcast::channel(16);
//         Self { sender }
//     }
// }

#[derive(Clone)]
pub struct BackendContext {
    // TODO: dont wrap connection, write something higher level
    db: Arc<Mutex<Connection>>,
    // TODO: make this generic
    harness: OpencodeHarness,
}

impl BackendContext {
    fn new(conn: Connection, harness: OpencodeHarness) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            harness,
        }
    }
}

pub trait Backend {
    async fn subscribe_projects(&self) -> Result<watch::Receiver<Vec<Project>>, BackendError>;
    async fn subscribe_project(
        &self,
        project_id: Uuid,
    ) -> Result<watch::Receiver<Option<Project>>, BackendError>;
    async fn subscribe_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<watch::Receiver<Vec<Session>>, BackendError>;
    async fn subscribe_session(
        &self,
        session_id: Uuid,
    ) -> Result<watch::Receiver<Option<Session>>, BackendError>;
    async fn list_projects(&self) -> Result<Vec<Project>, BackendError>;
    async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, BackendError>;
    async fn create_project(&self, project: Project) -> Result<Project, BackendError>;
    async fn update_project(&self, project: Project) -> Result<Project, BackendError>;
    async fn delete_project(&self, project_id: Uuid) -> Result<(), BackendError>;
    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, BackendError>;
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, BackendError>;
    async fn create_session(&self, session: Session) -> Result<Session, BackendError>;
    async fn update_session(&self, session: Session) -> Result<Session, BackendError>;
    async fn delete_session(&self, session_id: Uuid) -> Result<(), BackendError>;
}
