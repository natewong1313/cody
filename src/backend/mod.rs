use crate::backend::{
    data::{project::ProjectRepoError, session::SessionRepoError},
    db::Database,
    harness::opencode::OpencodeHarness,
    state::StateError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::watch;
use uuid::Uuid;

pub use data::project::Project;
pub use data::session::Session;
pub use grpc::spawn_backend;

mod data;
mod db;
mod grpc;
mod harness;
pub mod proto_utils;
mod state;

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
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
