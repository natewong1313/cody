use crate::backend::{Project, Session};

use super::{
    Database, DatabaseError, DatabaseStartupError, DatabaseTransaction,
    migrations::SQLITE_MIGRATIONS,
};
use rusqlite::{Connection, Row};
use std::sync::{Arc, Mutex, MutexGuard};

mod projects;
mod sessions;
#[cfg(test)]
mod test;

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

pub(super) fn check_returning_row_error(op: &'static str, err: rusqlite::Error) -> DatabaseError {
    match err {
        rusqlite::Error::QueryReturnedNoRows => DatabaseError::UnexpectedRowsAffected {
            op,
            expected: 1,
            actual: 0,
        },
        other => DatabaseError::SqliteQueryError(other),
    }
}

pub(super) fn assert_one_row_affected(
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

pub(super) fn row_to_project(row: &Row) -> Result<Project, rusqlite::Error> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        dir: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub(super) fn row_to_session(row: &Row) -> Result<Session, rusqlite::Error> {
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

    async fn list_projects(&self) -> Result<Vec<Project>, DatabaseError> {
        self.with_conn(projects::list_projects)
    }

    async fn get_project(&self, project_id: uuid::Uuid) -> Result<Option<Project>, DatabaseError> {
        self.with_conn(|conn| projects::get_project(conn, project_id))
    }

    async fn create_project(
        &self,
        project: Project,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Project, DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| projects::create_project(conn, &project))
    }

    async fn update_project(
        &self,
        project: Project,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Project, DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| projects::update_project(conn, &project))
    }

    async fn delete_project(
        &self,
        project_id: uuid::Uuid,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<(), DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| projects::delete_project(conn, project_id))
    }

    async fn list_sessions_by_project(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        self.with_conn(|conn| sessions::list_sessions_by_project(conn, project_id))
    }

    async fn get_session(&self, session_id: uuid::Uuid) -> Result<Option<Session>, DatabaseError> {
        self.with_conn(|conn| sessions::get_session(conn, session_id))
    }

    async fn create_session(
        &self,
        session: Session,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Session, DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| sessions::create_session(conn, &session))
    }

    async fn update_session(
        &self,
        session: Session,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Session, DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| sessions::update_session(conn, &session))
    }

    async fn delete_session(
        &self,
        session_id: uuid::Uuid,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<(), DatabaseError> {
        self.with_optional_tx_conn(tx, |conn| sessions::delete_session(conn, session_id))
    }
}
