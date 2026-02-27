use chrono::NaiveDateTime;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::DatabaseError,
    harness::Harness,
    proto_session,
    proto_utils::{format_naive_datetime, parse_naive_datetime, parse_uuid},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub show_in_gui: bool,
    pub name: String,
    pub harness_type: String,
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

impl From<Session> for proto_session::SessionModel {
    fn from(session: Session) -> Self {
        Self {
            id: session.id.to_string(),
            project_id: session.project_id.to_string(),
            show_in_gui: session.show_in_gui,
            name: session.name,
            created_at: format_naive_datetime(session.created_at),
            updated_at: format_naive_datetime(session.updated_at),
        }
    }
}

impl TryFrom<proto_session::SessionModel> for Session {
    type Error = Status;

    fn try_from(model: proto_session::SessionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: parse_uuid("session.id", &model.id)?,
            project_id: parse_uuid("session.project_id", &model.project_id)?,
            show_in_gui: model.show_in_gui,
            name: model.name,
            harness_type: "opencode".to_string(),
            created_at: parse_naive_datetime("session.created_at", &model.created_at)?,
            updated_at: parse_naive_datetime("session.updated_at", &model.updated_at)?,
        })
    }
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

        let project_dir = Some(project.dir.as_str());
        let harness_id = self
            .ctx
            .harness
            .create_session(session.clone(), project_dir)
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        let created = self.ctx.db.create_session(session.clone()).await?;
        if let Err(err) = self
            .ctx
            .db
            .set_session_harness_id(created.id, harness_id)
            .await
        {
            if let Err(delete_err) = self.ctx.db.delete_session(created.id).await {
                log::error!(
                    "failed to delete session {} after set_session_harness_id error: {}",
                    created.id,
                    delete_err
                );
            }
            return Err(SessionRepoError::Database(err));
        }

        Ok(created)
    }

    pub async fn update(&self, session: &Session) -> Result<Session, SessionRepoError> {
        Ok(self.ctx.db.update_session(session.clone()).await?)
    }

    pub async fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        self.ctx.db.delete_session(*session_id).await?;
        Ok(())
    }
}
