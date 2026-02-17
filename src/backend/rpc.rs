use crate::backend::Session;

use super::{BackendServer, Project, harness::Harness};
use tarpc::context::Context;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("database error: {0}")]
    Database(#[from] super::db::DatabaseError),
    #[error("project not found for session.project_id {0}")]
    ProjectNotFound(Uuid),
    #[error("harness error: {0}")]
    Harness(String),
}

#[tarpc::service]
pub trait BackendRpc {
    async fn list_projects() -> Result<Vec<Project>, RpcError>;
    async fn get_project(project_id: Uuid) -> Result<Option<Project>, RpcError>;
    async fn create_project(project: Project) -> Result<Project, RpcError>;
    async fn update_project(project: Project) -> Result<Project, RpcError>;
    async fn delete_project(project_id: Uuid) -> Result<(), RpcError>;

    async fn list_sessions_by_project(project_id: Uuid) -> Result<Vec<Session>, RpcError>;
    async fn get_session(session_id: Uuid) -> Result<Option<Session>, RpcError>;
    async fn create_session(session: Session) -> Result<Session, RpcError>;
    async fn update_session(session: Session) -> Result<Session, RpcError>;
    async fn delete_session(session_id: Uuid) -> Result<(), RpcError>;
}

impl BackendRpc for BackendServer {
    async fn list_projects(self, _: Context) -> Result<Vec<Project>, RpcError> {
        Ok(self.db.list_projects()?)
    }

    async fn get_project(self, _: Context, project_id: Uuid) -> Result<Option<Project>, RpcError> {
        Ok(self.db.get_project(&project_id)?)
    }

    async fn create_project(self, _: Context, project: Project) -> Result<Project, RpcError> {
        self.db.create_project(&project)?;
        Ok(project)
    }

    async fn update_project(self, _: Context, project: Project) -> Result<Project, RpcError> {
        self.db.update_project(&project)?;
        Ok(project)
    }

    async fn delete_project(self, _: Context, project_id: Uuid) -> Result<(), RpcError> {
        self.db.delete_project(&project_id)?;
        Ok(())
    }

    async fn list_sessions_by_project(
        self,
        _: Context,
        project_id: Uuid,
    ) -> Result<Vec<Session>, RpcError> {
        Ok(self.db.list_sessions_by_project(&project_id)?)
    }

    async fn get_session(self, _: Context, session_id: Uuid) -> Result<Option<Session>, RpcError> {
        Ok(self.db.get_session(&session_id)?)
    }

    async fn create_session(self, _: Context, session: Session) -> Result<Session, RpcError> {
        let project = self
            .db
            .get_project(&session.project_id)?
            .ok_or_else(|| RpcError::ProjectNotFound(session.project_id))?;

        self.harness
            .create_session(session.clone(), Some(project.dir.as_str()))
            .await
            .map_err(|e| RpcError::Harness(e.to_string()))?;

        self.db.create_session(&session)?;
        Ok(session)
    }

    async fn update_session(self, _: Context, session: Session) -> Result<Session, RpcError> {
        self.db.update_session(&session)?;
        Ok(session)
    }

    async fn delete_session(self, _: Context, session_id: Uuid) -> Result<(), RpcError> {
        self.db.delete_session(&session_id)?;
        Ok(())
    }
}
