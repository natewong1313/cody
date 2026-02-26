use crate::backend::repo::message::{Message, MessagePart};
use crate::backend::{Project, Session};

use super::{Database, DatabaseError, DatabaseStartupError, migrations::SQLITE_MIGRATIONS};
use tokio_rusqlite::{Connection as AsyncConnection, Error as AsyncError, rusqlite::Connection};
use uuid::Uuid;

mod messages;
mod projects;
mod sessions;
#[cfg(test)]
mod test;

pub fn now_utc_string() -> String {
    chrono::Utc::now().naive_utc().to_string()
}

pub fn check_returning_row_error(
    op: &'static str,
    err: tokio_rusqlite::rusqlite::Error,
) -> DatabaseError {
    match err {
        tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows => {
            DatabaseError::UnexpectedRowsAffected {
                op,
                expected: 1,
                actual: 0,
            }
        }
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

pub struct Sqlite {
    conn: AsyncConnection,
}

impl Sqlite {
    pub fn new() -> Result<Self, DatabaseStartupError> {
        let conn = Connection::open("./cody.db")?;
        Sqlite::new_with_conn(conn)
    }

    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Self, DatabaseStartupError> {
        let conn = Connection::open_in_memory()?;
        Sqlite::new_with_conn(conn)
    }

    fn new_with_conn(mut conn: Connection) -> Result<Self, DatabaseStartupError> {
        conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        SQLITE_MIGRATIONS.to_latest(&mut conn)?;
        Ok(Self { conn: conn.into() })
    }

    async fn with_conn<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, DatabaseError> + Send + 'static,
    ) -> Result<T, DatabaseError>
    where
        T: Send + 'static,
    {
        self.conn
            .call(|conn| f(conn))
            .await
            .map_err(|err| match err {
                AsyncError::ConnectionClosed => DatabaseError::ConnectionClosed,
                AsyncError::Close((_conn, err)) => DatabaseError::SqliteQueryError(err),
                AsyncError::Error(err) => err,
                _ => DatabaseError::ConnectionClosed,
            })
    }
}

impl Database for Sqlite {
    async fn list_projects(&self) -> Result<Vec<Project>, DatabaseError> {
        self.with_conn(projects::list_projects).await
    }

    async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
        self.with_conn(move |conn| projects::get_project(conn, project_id))
            .await
    }

    async fn create_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(move |conn| projects::create_project(conn, &project))
            .await
    }

    async fn update_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(move |conn| projects::update_project(conn, &project))
            .await
    }

    async fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| projects::delete_project(conn, project_id))
            .await
    }

    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        self.with_conn(move |conn| sessions::list_sessions_by_project(conn, project_id))
            .await
    }

    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
        self.with_conn(move |conn| sessions::get_session(conn, session_id))
            .await
    }

    async fn create_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(move |conn| sessions::create_session(conn, &session))
            .await
    }

    async fn update_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(move |conn| sessions::update_session(conn, &session))
            .await
    }

    async fn delete_session(&self, session_id: uuid::Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| sessions::delete_session(conn, session_id))
            .await
    }

    async fn set_session_harness_id(
        &self,
        session_id: Uuid,
        harness_id: String,
    ) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| sessions::set_session_harness_id(conn, session_id, &harness_id))
            .await
    }

    async fn get_session_harness_id(
        &self,
        session_id: Uuid,
    ) -> Result<Option<String>, DatabaseError> {
        self.with_conn(move |conn| sessions::get_session_harness_id(conn, session_id))
            .await
    }

    async fn get_session_id_by_harness_id(
        &self,
        harness_id: &str,
    ) -> Result<Option<Uuid>, DatabaseError> {
        let harness_id = harness_id.to_string();
        self.with_conn(move |conn| messages::get_session_id_by_harness_id(conn, &harness_id))
            .await
    }

    async fn upsert_session_message(&self, message: Message) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| messages::upsert_session_message(conn, &message))
            .await
    }

    async fn upsert_session_message_with_parts(
        &self,
        message: Message,
    ) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| messages::upsert_session_message_with_parts(conn, &message))
            .await
    }

    async fn ensure_session_message_exists(
        &self,
        session_id: Uuid,
        message_id: &str,
    ) -> Result<(), DatabaseError> {
        let message_id = message_id.to_string();
        self.with_conn(move |conn| {
            messages::ensure_session_message_exists(conn, session_id, &message_id)
        })
        .await
    }

    async fn mark_session_message_removed(
        &self,
        session_id: Uuid,
        message_id: &str,
    ) -> Result<(), DatabaseError> {
        let message_id = message_id.to_string();
        self.with_conn(move |conn| {
            messages::mark_session_message_removed(conn, session_id, &message_id)
        })
        .await
    }

    async fn upsert_session_message_part(
        &self,
        session_id: Uuid,
        part: MessagePart,
        delta: Option<String>,
    ) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| {
            messages::upsert_session_message_part(conn, session_id, &part, delta.as_deref())
        })
        .await
    }

    async fn list_session_messages(
        &self,
        session_id: Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<Message>, DatabaseError> {
        self.with_conn(move |conn| messages::list_session_messages(conn, session_id, limit))
            .await
    }
}
