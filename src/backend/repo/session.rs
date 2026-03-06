use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    BackendContext, db::DatabaseError, harness::Harness, models::session_model::SessionModel,
};

#[derive(Debug, Error)]
pub enum SessionRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("project not found for session.project_id {0}")]
    ProjectNotFound(Uuid),
    #[error("harness error: {0}")]
    Harness(String),
}

impl From<SessionRepoError> for tonic::Status {
    fn from(err: SessionRepoError) -> Self {
        match err {
            SessionRepoError::Database(e) => tonic::Status::internal(e.to_string()),
            SessionRepoError::ProjectNotFound(id) => {
                tonic::Status::not_found(format!("project not found: {id}"))
            }
            SessionRepoError::Harness(message) => tonic::Status::unavailable(message),
        }
    }
}

pub struct SessionRepo {
    ctx: BackendContext,
}

impl SessionRepo {
    pub fn new(ctx: BackendContext) -> Self {
        Self { ctx }
    }

    pub async fn list_by_project(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<SessionModel>, SessionRepoError> {
        Ok(self.ctx.db.list_sessions_by_project(*project_id).await?)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<SessionModel>, SessionRepoError> {
        Ok(self.ctx.db.get_session(*id).await?)
    }

    pub async fn create(&self, session: &SessionModel) -> Result<SessionModel, SessionRepoError> {
        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        let project_dir = Some(project.dir.as_str());
        let harness_session_id = self
            .ctx
            .harness
            .create_session(session.clone(), project_dir)
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        let mut created = session.clone();
        created.harness_session_id = harness_session_id;
        if created.dir.is_none() {
            created.dir = Some(project.dir);
        }

        Ok(self.ctx.db.create_session(created).await?)
    }

    pub async fn update(&self, session: &SessionModel) -> Result<SessionModel, SessionRepoError> {
        let mut updated = session.clone();
        if updated.harness_session_id.is_empty() {
            if let Some(existing) = self.ctx.db.get_session(updated.id).await? {
                updated.harness_session_id = existing.harness_session_id;
            }
        }

        Ok(self.ctx.db.update_session(updated).await?)
    }

    pub async fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        self.ctx.db.delete_session(*session_id).await?;
        Ok(())
    }
}
