use chrono::NaiveDateTime;
use thiserror::Error;
use uuid::Uuid;

use crate::backend::{BackendContext, db::DatabaseError, harness::Harness};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub show_in_gui: bool,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Error)]
pub enum SessionRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("project not found for session.project_id {0}")]
    ProjectNotFound(Uuid),
    #[error("harness error: {0}")]
    Harness(String),
}

pub struct SessionRepo<D>
where
    D: crate::backend::db::Database,
{
    ctx: BackendContext<D>,
}

impl<D> SessionRepo<D>
where
    D: crate::backend::db::Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn list_by_project(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<Session>, SessionRepoError> {
        self.ctx
            .db
            .list_sessions_by_project(*project_id)
            .await
            .map_err(SessionRepoError::from)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Session>, SessionRepoError> {
        self.ctx
            .db
            .get_session(*id)
            .await
            .map_err(SessionRepoError::from)
    }

    pub async fn create(&self, session: &Session) -> Result<Session, SessionRepoError> {
        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await
            .map_err(SessionRepoError::from)?
            .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        self.ctx
            .harness
            .create_session(session.clone(), Some(project.dir.as_str()))
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        self.ctx
            .db
            .create_session(session.clone())
            .await
            .map_err(SessionRepoError::from)
    }

    pub async fn update(&self, session: &Session) -> Result<Session, SessionRepoError> {
        self.ctx
            .db
            .update_session(session.clone())
            .await
            .map_err(SessionRepoError::from)
    }

    pub async fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        self.ctx
            .db
            .delete_session(*session_id)
            .await
            .map_err(SessionRepoError::from)
    }
}
