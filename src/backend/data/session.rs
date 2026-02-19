use chrono::Utc;
use rusqlite::OptionalExtension;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    backend::{
        BackendContext, Session,
        db::{DatabaseError, assert_one_row_affected, check_returning_row_error},
        harness::Harness,
    },
    with_db_conn,
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

impl From<rusqlite::Error> for SessionRepoError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(DatabaseError::from(err))
    }
}

pub struct SessionRepo {
    ctx: BackendContext,
}

impl SessionRepo {
    pub fn new(ctx: BackendContext) -> Self {
        Self { ctx }
    }

    pub fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<Session>, SessionRepoError> {
        let sessions = with_db_conn!(self, conn, {
            let mut stmt = conn
                .prepare("SELECT * FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC")?;
            let sessions = stmt
                .query_map([project_id], Session::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok::<Vec<Session>, DatabaseError>(sessions)
        })
        .map_err(SessionRepoError::from)?;

        Ok(sessions)
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Session>, SessionRepoError> {
        let session = with_db_conn!(self, conn, {
            let mut stmt = conn.prepare("SELECT * FROM sessions WHERE id = ?1")?;
            let session = stmt.query_row([id], Session::from_row).optional()?;
            Ok::<Option<Session>, DatabaseError>(session)
        })
        .map_err(SessionRepoError::from)?;

        Ok(session)
    }

    pub async fn create(&self, session: &Session) -> Result<Session, SessionRepoError> {
        let project_dir = with_db_conn!(self, conn, {
            let mut stmt = conn.prepare("SELECT dir FROM projects WHERE id = ?1")?;
            let dir = stmt
                .query_row([&session.project_id], |row| row.get::<_, String>(0))
                .optional()?;
            Ok::<Option<String>, DatabaseError>(dir)
        })
        .map_err(SessionRepoError::from)?
        .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        self.ctx
            .harness
            .create_session(session.clone(), Some(project_dir.as_str()))
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        let created = with_db_conn!(self, conn, {
            let created = conn.query_row(
                "INSERT INTO sessions (id, project_id, show_in_gui, name, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 RETURNING id, project_id, show_in_gui, name, created_at, updated_at",
                (
                    &session.id,
                    &session.project_id,
                    &session.show_in_gui,
                    &session.name,
                    &session.created_at,
                    &session.updated_at,
                ),
                Session::from_row,
            )?;
            Ok::<Session, DatabaseError>(created)
        })
        .map_err(SessionRepoError::from)?;

        Ok(created)
    }

    pub fn update(&self, session: &Session) -> Result<Session, SessionRepoError> {
        let updated = with_db_conn!(self, conn, {
            let updated = conn
                .query_row(
                    "UPDATE sessions
                     SET project_id = ?2, show_in_gui = ?3, name = ?4, updated_at = ?5
                     WHERE id = ?1
                     RETURNING id, project_id, show_in_gui, name, created_at, updated_at",
                    (
                        &session.id,
                        &session.project_id,
                        &session.show_in_gui,
                        &session.name,
                        Utc::now().naive_utc(),
                    ),
                    Session::from_row,
                )
                .map_err(|e| check_returning_row_error("update_session", e))?;
            Ok::<Session, DatabaseError>(updated)
        })
        .map_err(SessionRepoError::from)?;

        Ok(updated)
    }

    pub fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        with_db_conn!(self, conn, {
            let rows = conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
            assert_one_row_affected("delete_session", rows)
        })
        .map_err(SessionRepoError::from)?;

        Ok(())
    }
}
