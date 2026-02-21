use chrono::{NaiveDateTime, Utc};
use rusqlite::{OptionalExtension, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    backend::{
        BackendContext,
        db::{DatabaseError, assert_one_row_affected, check_returning_row_error},
        harness::Harness,
    },
    with_db_conn,
};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub show_in_gui: bool,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Session {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            project_id: row.get(1)?,
            show_in_gui: row.get(2)?,
            name: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
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

pub struct SessionRepo {
    ctx: BackendContext,
}

impl SessionRepo {
    pub fn new(ctx: BackendContext) -> Self {
        Self { ctx }
    }

    pub fn list(&self) -> Result<Vec<Session>, SessionRepoError> {
        with_db_conn!(self, conn, {
            (|| -> Result<Vec<Session>, DatabaseError> {
                let mut stmt = conn.prepare("SELECT * FROM sessions ORDER BY updated_at DESC")?;
                let sessions = stmt
                    .query_map([], Session::from_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(sessions)
            })()
        })
        .map_err(SessionRepoError::from)
    }

    pub fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<Session>, SessionRepoError> {
        with_db_conn!(self, conn, {
            (|| -> Result<Vec<Session>, DatabaseError> {
                let mut stmt = conn.prepare(
                    "SELECT * FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC",
                )?;
                let sessions = stmt
                    .query_map([project_id], Session::from_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(sessions)
            })()
        })
        .map_err(SessionRepoError::from)
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Session>, SessionRepoError> {
        with_db_conn!(self, conn, {
            (|| -> Result<Option<Session>, DatabaseError> {
                let mut stmt = conn.prepare("SELECT * FROM sessions WHERE id = ?1")?;
                let session = stmt.query_row([id], Session::from_row).optional()?;
                Ok(session)
            })()
        })
        .map_err(SessionRepoError::from)
    }

    pub async fn create(&self, session: &Session) -> Result<Session, SessionRepoError> {
        let project_dir = with_db_conn!(self, conn, {
            (|| -> Result<Option<String>, DatabaseError> {
                let mut stmt = conn.prepare("SELECT dir FROM projects WHERE id = ?1")?;
                let dir = stmt
                    .query_row([&session.project_id], |row| row.get::<_, String>(0))
                    .optional()?;
                Ok(dir)
            })()
        })
        .map_err(SessionRepoError::from)?
        .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        self.ctx
            .harness
            .create_session(session.clone(), Some(project_dir.as_str()))
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        with_db_conn!(self, conn, {
            (|| -> Result<Session, DatabaseError> {
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
                Ok(created)
            })()
        })
        .map_err(SessionRepoError::from)
    }

    pub fn update(&self, session: &Session) -> Result<Session, SessionRepoError> {
        with_db_conn!(self, conn, {
            (|| -> Result<Session, DatabaseError> {
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
                Ok(updated)
            })()
        })
        .map_err(SessionRepoError::from)
    }

    pub fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        with_db_conn!(self, conn, {
            (|| -> Result<(), DatabaseError> {
                let rows = conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
                assert_one_row_affected("delete_session", rows)
            })()
        })
        .map_err(SessionRepoError::from)
    }
}
