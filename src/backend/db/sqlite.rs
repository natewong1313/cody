use crate::backend::{Project, Session};

use super::{
    Database, DatabaseError, DatabaseStartupError, DatabaseTransaction,
    migrations::SQLITE_MIGRATIONS,
};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Row};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct Sqlite {
    conn: Arc<Mutex<Connection>>,
}

pub struct SqliteTransaction<'a> {
    conn: MutexGuard<'a, Connection>,
    finished: bool,
}

impl DatabaseTransaction for SqliteTransaction<'_> {
    fn commit(&mut self) -> Result<(), DatabaseError> {
        if self.finished {
            return Ok(());
        }
        self.conn.execute_batch("COMMIT;")?;
        self.finished = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), DatabaseError> {
        if self.finished {
            return Ok(());
        }
        self.conn.execute_batch("ROLLBACK;")?;
        self.finished = true;
        Ok(())
    }
}

impl Drop for SqliteTransaction<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let _ = self.conn.execute_batch("ROLLBACK;");
        self.finished = true;
    }
}

impl Sqlite {
    pub fn new() -> Result<Self, DatabaseStartupError> {
        let conn = Connection::open("./cody.db")?;
        Sqlite::new_with_conn(conn)
    }

    pub fn new_in_memory() -> Result<Self, DatabaseStartupError> {
        let conn = Connection::open_in_memory()?;
        Sqlite::new_with_conn(conn)
    }

    fn new_with_conn(mut conn: Connection) -> Result<Self, DatabaseStartupError> {
        conn.pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        SQLITE_MIGRATIONS.to_latest(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn with_optional_tx_conn<T>(
        &self,
        tx: Option<&mut SqliteTransaction<'_>>,
        f: impl FnOnce(&Connection) -> Result<T, DatabaseError>,
    ) -> Result<T, DatabaseError> {
        match tx {
            Some(tx) => f(&tx.conn),
            None => self.with_conn(f),
        }
    }

    fn with_conn<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, DatabaseError>,
    ) -> Result<T, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::PoisonedLock)?;
        f(&conn)
    }

    pub async fn create_project(&self, project: Project) -> Result<Project, DatabaseError> {
        <Self as Database>::create_project(self, project, None).await
    }

    pub async fn update_project(&self, project: Project) -> Result<Project, DatabaseError> {
        <Self as Database>::update_project(self, project, None).await
    }

    pub async fn delete_project(&self, project_id: uuid::Uuid) -> Result<(), DatabaseError> {
        <Self as Database>::delete_project(self, project_id, None).await
    }

    pub async fn create_session(&self, session: Session) -> Result<Session, DatabaseError> {
        <Self as Database>::create_session(self, session, None).await
    }

    pub async fn update_session(&self, session: Session) -> Result<Session, DatabaseError> {
        <Self as Database>::update_session(self, session, None).await
    }

    pub async fn delete_session(&self, session_id: uuid::Uuid) -> Result<(), DatabaseError> {
        <Self as Database>::delete_session(self, session_id, None).await
    }
}

/// Helper function to make sure updates are updating
pub fn check_returning_row_error(op: &'static str, err: rusqlite::Error) -> DatabaseError {
    match err {
        rusqlite::Error::QueryReturnedNoRows => DatabaseError::UnexpectedRowsAffected {
            op,
            expected: 1,
            actual: 0,
        },
        other => DatabaseError::SqliteQueryError(other),
    }
}

/// Helper function to make sure rows are actually being deleted
pub fn assert_one_row_affected(
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

fn row_to_project(row: &Row) -> Result<Project, rusqlite::Error> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        dir: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn row_to_session(row: &Row) -> Result<Session, rusqlite::Error> {
    Ok(Session {
        id: row.get(0)?,
        project_id: row.get(1)?,
        show_in_gui: row.get(2)?,
        name: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

impl Database for Sqlite {
    type Transaction<'a>
        = SqliteTransaction<'a>
    where
        Self: 'a;

    async fn begin_transaction(&self) -> Result<Self::Transaction<'_>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::PoisonedLock)?;
        conn.execute_batch("BEGIN IMMEDIATE;")?;
        Ok(SqliteTransaction {
            conn,
            finished: false,
        })
    }

    async fn list_projects(&self) -> Result<Vec<crate::backend::Project>, super::DatabaseError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, dir, created_at, updated_at FROM projects ORDER BY updated_at DESC",
            )?;
            let projects = stmt
                .query_map([], row_to_project)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(projects)
        })
    }

    async fn get_project(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Option<crate::backend::Project>, super::DatabaseError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, dir, created_at, updated_at FROM projects WHERE id = ?1",
            )?;
            let project = stmt.query_row([project_id], row_to_project).optional()?;
            Ok(project)
        })
    }

    async fn create_project(
        &self,
        project: crate::backend::Project,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<crate::backend::Project, super::DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| {
            let created = conn.query_row(
                "INSERT INTO projects (id, name, dir, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 RETURNING id, name, dir, created_at, updated_at",
                (
                    &project.id,
                    &project.name,
                    &project.dir,
                    &project.created_at,
                    &project.updated_at,
                ),
                row_to_project,
            )?;
            Ok(created)
        })
    }

    async fn update_project(
        &self,
        project: crate::backend::Project,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<crate::backend::Project, super::DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| {
            let updated = conn
                .query_row(
                    "UPDATE projects
                     SET name = ?2, dir = ?3, updated_at = ?4
                     WHERE id = ?1
                     RETURNING id, name, dir, created_at, updated_at",
                    (
                        &project.id,
                        &project.name,
                        &project.dir,
                        Utc::now().naive_utc(),
                    ),
                    row_to_project,
                )
                .map_err(|e| check_returning_row_error("update_project", e))?;
            Ok(updated)
        })
    }

    async fn delete_project(
        &self,
        project_id: uuid::Uuid,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<(), super::DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| {
            let rows = conn.execute("DELETE FROM projects WHERE id = ?1", [project_id])?;
            assert_one_row_affected("delete_project", rows)
        })
    }

    async fn list_sessions_by_project(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Vec<crate::backend::Session>, super::DatabaseError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_id, show_in_gui, name, created_at, updated_at
                 FROM sessions
                 WHERE project_id = ?1
                 ORDER BY updated_at DESC",
            )?;
            let sessions = stmt
                .query_map([project_id], row_to_session)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(sessions)
        })
    }

    async fn get_session(
        &self,
        session_id: uuid::Uuid,
    ) -> Result<Option<crate::backend::Session>, super::DatabaseError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, project_id, show_in_gui, name, created_at, updated_at
                 FROM sessions
                 WHERE id = ?1",
            )?;
            let session = stmt.query_row([session_id], row_to_session).optional()?;
            Ok(session)
        })
    }

    async fn create_session(
        &self,
        session: crate::backend::Session,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<crate::backend::Session, super::DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| {
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
                row_to_session,
            )?;
            Ok(created)
        })
    }

    async fn update_session(
        &self,
        session: crate::backend::Session,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<crate::backend::Session, super::DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| {
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
                    row_to_session,
                )
                .map_err(|e| check_returning_row_error("update_session", e))?;
            Ok(updated)
        })
    }

    async fn delete_session(
        &self,
        session_id: uuid::Uuid,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<(), super::DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| {
            let rows = conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
            assert_one_row_affected("delete_session", rows)
        })
    }
}
