use crate::backend::Session;

use super::{BackendEvent, BackendServer, Project, harness::Harness};
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
        let created = self.db.create_project(&project)?;
        self.emit_event(BackendEvent::ProjectUpserted(created.clone()));
        Ok(created)
    }

    async fn update_project(self, _: Context, project: Project) -> Result<Project, RpcError> {
        let updated = self.db.update_project(&project)?;
        self.emit_event(BackendEvent::ProjectUpserted(updated.clone()));
        Ok(updated)
    }

    async fn delete_project(self, _: Context, project_id: Uuid) -> Result<(), RpcError> {
        self.db.delete_project(&project_id)?;
        self.emit_event(BackendEvent::ProjectDeleted(project_id));
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

        let created = self.db.create_session(&session)?;
        self.emit_event(BackendEvent::SessionUpserted(created.clone()));
        Ok(created)
    }

    async fn update_session(self, _: Context, session: Session) -> Result<Session, RpcError> {
        let updated = self.db.update_session(&session)?;
        self.emit_event(BackendEvent::SessionUpserted(updated.clone()));
        Ok(updated)
    }

    async fn delete_session(self, _: Context, session_id: Uuid) -> Result<(), RpcError> {
        self.db.delete_session(&session_id)?;
        self.emit_event(BackendEvent::SessionDeleted(session_id));
        Ok(())
    }
}
