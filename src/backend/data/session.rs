use chrono::NaiveDateTime;
use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{DatabaseError, DatabaseTransaction},
    harness::Harness,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        Ok(self.ctx.db.list_sessions_by_project(*project_id).await?)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Session>, SessionRepoError> {
        Ok(self.ctx.db.get_session(*id).await?)
    }

    pub async fn create(&self, session: &Session) -> Result<Session, SessionRepoError> {
        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        let mut tx = self.ctx.db.begin_transaction().await?;

        let created = self
            .ctx
            .db
            .create_session(session.clone(), Some(&mut tx))
            .await?;

        let project_dir = Some(project.dir.as_str());
        if let Err(e) = self
            .ctx
            .harness
            .create_session(session.clone(), project_dir)
            .await
        {
            tx.rollback()?;
            return Err(SessionRepoError::Harness(e.to_string()));
        }

        tx.commit()?;
        Ok(created)
    }

    pub async fn update(&self, session: &Session) -> Result<Session, SessionRepoError> {
        Ok(self.ctx.db.update_session(session.clone(), None).await?)
    }

    pub async fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        self.ctx.db.delete_session(*session_id, None).await?;
        Ok(())
    }
}
