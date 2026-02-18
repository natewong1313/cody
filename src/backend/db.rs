use crate::backend::{Project, Session, db_migrations::MIGRATIONS};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Error, Debug)]
pub enum DatabaseStartupError {
    #[error("Error establishing connection {0}")]
    Connection(#[from] rusqlite::Error),
    #[error("Error migrating db {0}")]
    Migration(#[from] rusqlite_migration::Error),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Generic database error {0}")]
    QueryError(#[from] rusqlite::Error),
    #[error("Db conn lock poisoned")]
    PoisonedLock,
    #[error("{op} unexpected rows affected, expected {expected} got {actual}")]
    UnexpectedRowsAffected {
        op: &'static str,
        expected: usize,
        actual: usize,
    },
}

/// Grabs a database connection or returns an error if its mutex lock is poisoned
macro_rules! with_conn {
    ($self:expr, $conn:ident, $body:block) => {{
        let $conn = $self.conn.lock().map_err(|_| DatabaseError::PoisonedLock)?;
        $body
    }};
}

impl Database {
    pub fn new() -> Result<Self, DatabaseStartupError> {
        let mut conn = Connection::open("./cody.db")?;
        conn.pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        MIGRATIONS.to_latest(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Helper function to make sure rows are actually being updated/deleted
    fn assert_one_row_affected(
        &self,
        op: &'static str,
        rows_affected: usize,
    ) -> Result<(), DatabaseError> {
        if rows_affected == 1 {
            Ok(())
        } else {
            Err(DatabaseError::UnexpectedRowsAffected {
                op,
                expected: 1,
                actual: rows_affected,
            })
        }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt = conn.prepare(
                "SELECT id, name, dir, created_at, updated_at FROM projects ORDER BY updated_at DESC",
            )?;
            let projects = stmt
                .query_map([], Project::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(projects)
        })
    }

    pub fn get_project(&self, id: &Uuid) -> Result<Option<Project>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt = conn.prepare(
                "SELECT id, name, dir, created_at, updated_at FROM projects WHERE id = ?1",
            )?;
            let project = stmt.query_row([id], Project::from_row).optional()?;
            Ok(project)
        })
    }

    pub fn create_project(&self, project: &Project) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "INSERT INTO projects (id, name, dir, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &project.id,
                    &project.name,
                    &project.dir,
                    &project.created_at,
                    &project.updated_at,
                ),
            ).map(|rows| {self.assert_one_row_affected("create_project", rows)})?
        })
    }

    pub fn update_project(&self, project: &Project) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "UPDATE projects SET name = ?2, dir = ?3, updated_at = ?4 WHERE id = ?1",
                (
                    &project.id,
                    &project.name,
                    &project.dir,
                    Utc::now().naive_utc(),
                ),
            )
            .map(|rows| self.assert_one_row_affected("update_project", rows))?
        })
    }

    pub fn delete_project(&self, project_id: &Uuid) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute("DELETE FROM projects WHERE id = ?1", [project_id])
                .map(|rows| self.assert_one_row_affected("delete_project", rows))?
        })
    }

    pub fn list_sessions_by_project(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt = conn
                .prepare("SELECT * FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC")?;
            let sessions = stmt
                .query_map([project_id], Session::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(sessions)
        })
    }

    pub fn get_session(&self, id: &Uuid) -> Result<Option<Session>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt = conn.prepare("SELECT * FROM sessions WHERE id = ?1")?;
            let session = stmt.query_row([id], Session::from_row).optional()?;
            Ok(session)
        })
    }

    pub fn create_session(&self, session: &Session) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "INSERT INTO sessions (id, project_id, show_in_gui, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &session.id,
                    &session.project_id,
                    &session.show_in_gui,
                    &session.name,
                    &session.created_at,
                    &session.updated_at,
                ),
            ).map(|rows| self.assert_one_row_affected("create_session", rows))?
        })
    }

    pub fn update_session(&self, session: &Session) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "UPDATE sessions SET project_id = ?2, show_in_gui = ?3, name = ?4, updated_at = ?5 WHERE id = ?1",
                (
                    &session.id,
                    &session.project_id,
                    &session.show_in_gui,
                    &session.name,
                    Utc::now().naive_utc(),
                ),
            ).map(|rows| self.assert_one_row_affected("update_session", rows))?
        })
    }

    pub fn delete_session(&self, session_id: &Uuid) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])
                .map(|rows| self.assert_one_row_affected("delete_session", rows))?
        })
    }
}
