use crate::backend::{Project, Session};
use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "CREATE TABLE projects (
            id BLOB CHECK(length(id) = 16),
            name TEXT NOT NULL,
            dir TEXT NOT NULL
        );",
    ),
    M::up(
        "CREATE TABLE sessions (
            id BLOB CHECK(length(id) = 16),
            project_id BLOB CHECK(length(project_id) = 16) REFERENCES projects(id),
            name TEXT NOT NULL
        );",
    ),
    M::up(
        "CREATE TABLE sessions_new (
            id BLOB CHECK(length(id) = 16),
            project_id BLOB CHECK(length(project_id) = 16) REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL
        );

        INSERT INTO sessions_new (id, project_id, name)
        SELECT s.id, s.project_id, s.name
        FROM sessions s
        JOIN projects p ON p.id = s.project_id;

        DROP TABLE sessions;
        ALTER TABLE sessions_new RENAME TO sessions;",
    ),
];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

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
}

macro_rules! with_conn {
    ($self:expr, $conn:ident, $body:block) => {{
        let $conn = $self.conn.lock().map_err(|_| DatabaseError::PoisonedLock)?;
        $body
    }};
}

impl Database {
    pub fn new() -> Result<Self, DatabaseStartupError> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        MIGRATIONS.to_latest(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt = conn.prepare("SELECT id, name, dir FROM projects")?;
            let projects = stmt
                .query_map([], Project::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(projects)
        })
    }

    pub fn get_project(&self, id: &Uuid) -> Result<Option<Project>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt = conn.prepare("SELECT id, name, dir FROM projects WHERE id = ?1")?;
            let project = stmt.query_row([id], Project::from_row).optional()?;
            Ok(project)
        })
    }

    pub fn create_project(&self, project: &Project) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "INSERT INTO projects (id, name, dir) VALUES (?1, ?2, ?3)",
                (&project.id, &project.name, &project.dir),
            )?;
            Ok(())
        })
    }

    pub fn update_project(&self, project: &Project) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "UPDATE projects SET name = ?2, dir = ?3 WHERE id = ?1",
                (&project.id, &project.name, &project.dir),
            )?;
            Ok(())
        })
    }

    pub fn delete_project(&self, project_id: &Uuid) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute("DELETE FROM projects WHERE id = ?1", [project_id])?;
            Ok(())
        })
    }

    pub fn list_sessions_by_project(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt =
                conn.prepare("SELECT id, project_id, name FROM sessions WHERE project_id = ?1")?;
            let sessions = stmt
                .query_map([project_id], Session::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(sessions)
        })
    }

    pub fn get_session(&self, id: &Uuid) -> Result<Option<Session>, DatabaseError> {
        with_conn!(self, conn, {
            let mut stmt =
                conn.prepare("SELECT id, project_id, name FROM sessions WHERE id = ?1")?;
            let session = stmt.query_row([id], Session::from_row).optional()?;
            Ok(session)
        })
    }

    pub fn create_session(&self, session: &Session) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "INSERT INTO sessions (id, project_id, name) VALUES (?1, ?2, ?3)",
                (&session.id, &session.project_id, &session.name),
            )?;
            Ok(())
        })
    }

    pub fn update_session(&self, session: &Session) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute(
                "UPDATE sessions SET project_id = ?2, name = ?3 WHERE id = ?1",
                (&session.id, &session.project_id, &session.name),
            )?;
            Ok(())
        })
    }

    pub fn delete_session(&self, session_id: &Uuid) -> Result<(), DatabaseError> {
        with_conn!(self, conn, {
            conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
            Ok(())
        })
    }
}
