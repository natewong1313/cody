use crate::backend::{
    Project, Session,
    repo::assistant_message::{AssistantMessage, AssistantMessagePart},
    repo::message::Message,
    repo::user_message::UserMessage,
    repo::user_message_part::UserMessagePart,
};

use super::{Database, DatabaseError, DatabaseStartupError, migrations::SQLITE_MIGRATIONS};
use tokio_rusqlite::{Connection as AsyncConnection, Error as AsyncError, rusqlite::Connection};
use uuid::Uuid;

mod assistant_message;
mod assistant_message_part;
mod message;
mod project;
#[cfg(test)]
mod project_test;
mod session;
#[cfg(test)]
mod session_test;
mod user_message;
mod user_message_part;

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

pub fn expect_one_returned_row<T>(
    op: &'static str,
    mut rows: impl Iterator<Item = serde_rusqlite::Result<T>>,
) -> Result<T, DatabaseError> {
    rows.next()
        .transpose()?
        .ok_or(DatabaseError::UnexpectedRowsAffected {
            op,
            expected: 1,
            actual: 0,
        })
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
        self.with_conn(project::list).await
    }

    async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
        self.with_conn(move |conn| project::get(conn, project_id))
            .await
    }

    async fn create_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(move |conn| project::create(conn, &project))
            .await
    }

    async fn update_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(move |conn| project::update(conn, &project))
            .await
    }

    async fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| project::delete(conn, project_id))
            .await
    }

    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        self.with_conn(move |conn| session::list_by_project(conn, project_id))
            .await
    }

    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
        self.with_conn(move |conn| session::get(conn, session_id))
            .await
    }

    async fn create_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(move |conn| session::create(conn, &session))
            .await
    }

    async fn update_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(move |conn| session::update(conn, &session))
            .await
    }

    async fn delete_session(&self, session_id: uuid::Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| session::delete(conn, session_id))
            .await
    }

    async fn list_messages_by_session(
        &self,
        session_id: Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, DatabaseError> {
        self.with_conn(move |conn| message::list_messages_by_session(conn, session_id, limit))
            .await
    }

    async fn list_user_messages_by_session(
        &self,
        session_id: Uuid,
        limit: u32,
    ) -> Result<Vec<UserMessage>, DatabaseError> {
        self.with_conn(move |conn| user_message::list_by_session(conn, session_id, limit))
            .await
    }

    async fn get_user_message(
        &self,
        user_message_id: Uuid,
    ) -> Result<Option<UserMessage>, DatabaseError> {
        self.with_conn(move |conn| user_message::get(conn, user_message_id))
            .await
    }

    async fn create_user_message(
        &self,
        user_message_item: UserMessage,
    ) -> Result<UserMessage, DatabaseError> {
        self.with_conn(move |conn| user_message::create(conn, &user_message_item))
            .await
    }

    async fn update_user_message(
        &self,
        user_message_item: UserMessage,
    ) -> Result<UserMessage, DatabaseError> {
        self.with_conn(move |conn| user_message::update(conn, &user_message_item))
            .await
    }

    async fn delete_user_message(&self, user_message_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| user_message::delete(conn, user_message_id))
            .await
    }

    async fn get_user_message_part(
        &self,
        part_id: Uuid,
    ) -> Result<Option<UserMessagePart>, DatabaseError> {
        self.with_conn(move |conn| user_message_part::get(conn, part_id))
            .await
    }

    async fn create_user_message_part(
        &self,
        part: UserMessagePart,
    ) -> Result<UserMessagePart, DatabaseError> {
        self.with_conn(move |conn| user_message_part::create(conn, &part))
            .await
    }

    async fn update_user_message_part(
        &self,
        part: UserMessagePart,
    ) -> Result<UserMessagePart, DatabaseError> {
        self.with_conn(move |conn| user_message_part::update(conn, &part))
            .await
    }

    async fn delete_user_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| user_message_part::delete(conn, part_id))
            .await
    }

    async fn get_assistant_message(
        &self,
        assistant_message_id: Uuid,
    ) -> Result<Option<AssistantMessage>, DatabaseError> {
        self.with_conn(move |conn| assistant_message::get(conn, assistant_message_id))
            .await
    }

    async fn get_assistant_message_by_harness_id(
        &self,
        session_id: Uuid,
        harness_message_id: String,
    ) -> Result<Option<AssistantMessage>, DatabaseError> {
        self.with_conn(move |conn| {
            assistant_message::get_by_harness_id(conn, session_id, &harness_message_id)
        })
        .await
    }

    async fn create_assistant_message(
        &self,
        assistant_message_item: AssistantMessage,
    ) -> Result<AssistantMessage, DatabaseError> {
        self.with_conn(move |conn| assistant_message::create(conn, &assistant_message_item))
            .await
    }

    async fn update_assistant_message(
        &self,
        assistant_message_item: AssistantMessage,
    ) -> Result<AssistantMessage, DatabaseError> {
        self.with_conn(move |conn| assistant_message::update(conn, &assistant_message_item))
            .await
    }

    async fn delete_assistant_message(
        &self,
        assistant_message_id: Uuid,
    ) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| assistant_message::delete(conn, assistant_message_id))
            .await
    }

    async fn get_assistant_message_part(
        &self,
        part_id: Uuid,
    ) -> Result<Option<AssistantMessagePart>, DatabaseError> {
        self.with_conn(move |conn| assistant_message_part::get(conn, part_id))
            .await
    }

    async fn get_assistant_message_part_by_harness_id(
        &self,
        assistant_message_id: Uuid,
        harness_part_id: String,
    ) -> Result<Option<AssistantMessagePart>, DatabaseError> {
        self.with_conn(move |conn| {
            assistant_message_part::get_by_harness_id(conn, assistant_message_id, &harness_part_id)
        })
        .await
    }

    async fn create_assistant_message_part(
        &self,
        part: AssistantMessagePart,
    ) -> Result<AssistantMessagePart, DatabaseError> {
        self.with_conn(move |conn| assistant_message_part::create(conn, &part))
            .await
    }

    async fn update_assistant_message_part(
        &self,
        part: AssistantMessagePart,
    ) -> Result<AssistantMessagePart, DatabaseError> {
        self.with_conn(move |conn| assistant_message_part::update(conn, &part))
            .await
    }

    async fn delete_assistant_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| assistant_message_part::delete(conn, part_id))
            .await
    }
}
