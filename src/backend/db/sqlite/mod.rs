use crate::backend::{Project, Session};

use super::{Database, DatabaseError, DatabaseStartupError, migrations::SQLITE_MIGRATIONS};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod projects;
mod sessions;
#[cfg(test)]
mod test;

pub struct Sqlite {
    conn: Arc<Mutex<Connection>>,
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

    fn with_conn<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, DatabaseError>,
    ) -> Result<T, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::PoisonedLock)?;
        f(&conn)
    }
}

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

impl Database for Sqlite {
    fn list_projects(&self) -> Result<Vec<Project>, DatabaseError> {
        self.with_conn(projects::list_projects)
    }

    fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
        self.with_conn(|conn| projects::get_project(conn, project_id))
    }

    fn create_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(|conn| projects::create_project(conn, &project))
    }

    fn update_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(|conn| projects::update_project(conn, &project))
    }

    fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(|conn| projects::delete_project(conn, project_id))
    }

    fn list_sessions_by_project(&self, project_id: Uuid) -> Result<Vec<Session>, DatabaseError> {
        self.with_conn(|conn| sessions::list_sessions_by_project(conn, project_id))
    }

    fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
        self.with_conn(|conn| sessions::get_session(conn, session_id))
    }

    fn create_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(|conn| sessions::create_session(conn, &session))
    }

    fn update_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(|conn| sessions::update_session(conn, &session))
    }

    fn delete_session(&self, session_id: uuid::Uuid) -> Result<(), DatabaseError> {
        self.with_conn(|conn| sessions::delete_session(conn, session_id))
    }
}
