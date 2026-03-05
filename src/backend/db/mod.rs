use thiserror::Error;
use tokio_rusqlite::Connection;
use uuid::Uuid;

use crate::backend::{
    Project, Session,
    db::migrations::SQLITE_MIGRATIONS,
    repo::{
        assistant_message::{AssistantMessage, AssistantMessagePart},
        message::Message,
        user_message::UserMessage,
        user_message_part::UserMessagePart,
    },
};

mod assistant_message_part_table;
mod assistant_message_table;
mod message_table;
mod migrations;
mod project_table;
mod session_table;
mod user_message_part_table;
mod user_message_table;

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
        other => DatabaseError::UnknownError(other),
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

#[derive(Error, Debug)]
pub enum DatabaseStartupError {
    #[error("Error establishing sqlite connection {0}")]
    SqliteConnection(#[from] tokio_rusqlite::rusqlite::Error),
    #[error("Error migrating sqlite {0}")]
    SqliteMigration(#[from] rusqlite_migration::Error),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Db connection closed")]
    ConnectionClosed,
    #[error("{op} unexpected rows affected, expected {expected} got {actual}")]
    UnexpectedRowsAffected {
        op: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("Serde rusqlite error {0}")]
    SerdeError(#[from] serde_rusqlite::Error),
    #[error("Unknown query error: {0}")]
    UnknownQueryError(#[from] tokio_rusqlite::rusqlite::Error),
    #[error("Unknown database error: {0}")]
    UnknownError(tokio_rusqlite::rusqlite::Error),
    #[error("Db connection failed to close: {0}")]
    FailedClose(tokio_rusqlite::rusqlite::Error),
}
impl From<tokio_rusqlite::Error<DatabaseError>> for DatabaseError {
    fn from(err: tokio_rusqlite::Error<DatabaseError>) -> Self {
        match err {
            tokio_rusqlite::Error::Error(db_err) => db_err,
            tokio_rusqlite::Error::ConnectionClosed => DatabaseError::ConnectionClosed,
            tokio_rusqlite::Error::Close((_, e)) => DatabaseError::FailedClose(e),
            _other => DatabaseError::UnknownError(tokio_rusqlite::rusqlite::Error::InvalidQuery),
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new() -> Result<Self, DatabaseStartupError> {
        let conn = Connection::open("./cody.db").await?;
        Database::new_with_conn(conn).await
    }

    #[allow(dead_code)]
    pub async fn new_in_memory() -> Result<Self, DatabaseStartupError> {
        let conn = Connection::open_in_memory().await?;
        Database::new_with_conn(conn).await
    }

    async fn new_with_conn(conn: Connection) -> Result<Self, DatabaseStartupError> {
        conn.call_unwrap(|conn| {
            conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            SQLITE_MIGRATIONS.to_latest(conn)
        })
        .await?;
        Ok(Self { conn: conn.into() })
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, DatabaseError> {
        Ok(self.conn.call(|conn| project_table::list(conn)).await?)
    }

    pub async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| project_table::get(conn, project_id))
            .await?)
    }

    pub async fn create_project(&self, project: Project) -> Result<Project, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| project_table::create(conn, &project))
            .await?)
    }

    pub async fn update_project(&self, project: Project) -> Result<Project, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| project_table::update(conn, &project))
            .await?)
    }

    pub async fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| project_table::delete(conn, project_id))
            .await?)
    }

    pub async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| session_table::list_by_project(conn, project_id))
            .await?)
    }

    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| session_table::get(conn, session_id))
            .await?)
    }

    pub async fn create_session(&self, session: Session) -> Result<Session, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| session_table::create(conn, &session))
            .await?)
    }

    pub async fn update_session(&self, session: Session) -> Result<Session, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| session_table::update(conn, &session))
            .await?)
    }

    pub async fn delete_session(&self, session_id: Uuid) -> Result<(), DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| session_table::delete(conn, session_id))
            .await?)
    }

    pub async fn list_messages_by_session(
        &self,
        session_id: Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| message_table::list_messages_by_session(conn, session_id, limit))
            .await?)
    }

    pub async fn list_user_messages_by_session(
        &self,
        session_id: Uuid,
        limit: u32,
    ) -> Result<Vec<UserMessage>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_table::list_by_session(conn, session_id, limit))
            .await?)
    }

    pub async fn get_user_message(
        &self,
        user_message_id: Uuid,
    ) -> Result<Option<UserMessage>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_table::get(conn, user_message_id))
            .await?)
    }

    pub async fn create_user_message(
        &self,
        user_message_item: UserMessage,
    ) -> Result<UserMessage, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_table::create(conn, &user_message_item))
            .await?)
    }

    pub async fn update_user_message(
        &self,
        user_message_item: UserMessage,
    ) -> Result<UserMessage, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_table::update(conn, &user_message_item))
            .await?)
    }

    pub async fn delete_user_message(&self, user_message_id: Uuid) -> Result<(), DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_table::delete(conn, user_message_id))
            .await?)
    }

    pub async fn get_user_message_part(
        &self,
        part_id: Uuid,
    ) -> Result<Option<UserMessagePart>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_part_table::get(conn, part_id))
            .await?)
    }

    pub async fn create_user_message_part(
        &self,
        part: UserMessagePart,
    ) -> Result<UserMessagePart, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_part_table::create(conn, &part))
            .await?)
    }

    pub async fn update_user_message_part(
        &self,
        part: UserMessagePart,
    ) -> Result<UserMessagePart, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_part_table::update(conn, &part))
            .await?)
    }

    pub async fn delete_user_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| user_message_part_table::delete(conn, part_id))
            .await?)
    }

    pub async fn get_assistant_message(
        &self,
        assistant_message_id: Uuid,
    ) -> Result<Option<AssistantMessage>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_table::get(conn, assistant_message_id))
            .await?)
    }

    pub async fn get_assistant_message_by_harness_id(
        &self,
        session_id: Uuid,
        harness_message_id: String,
    ) -> Result<Option<AssistantMessage>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| {
                assistant_message_table::get_by_harness_id(conn, session_id, &harness_message_id)
            })
            .await?)
    }

    pub async fn create_assistant_message(
        &self,
        assistant_message_item: AssistantMessage,
    ) -> Result<AssistantMessage, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_table::create(conn, &assistant_message_item))
            .await?)
    }

    pub async fn update_assistant_message(
        &self,
        assistant_message_item: AssistantMessage,
    ) -> Result<AssistantMessage, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_table::update(conn, &assistant_message_item))
            .await?)
    }

    pub async fn delete_assistant_message(
        &self,
        assistant_message_id: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_table::delete(conn, assistant_message_id))
            .await?)
    }

    pub async fn get_assistant_message_part(
        &self,
        part_id: Uuid,
    ) -> Result<Option<AssistantMessagePart>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_part_table::get(conn, part_id))
            .await?)
    }

    pub async fn get_assistant_message_part_by_harness_id(
        &self,
        assistant_message_id: Uuid,
        harness_part_id: String,
    ) -> Result<Option<AssistantMessagePart>, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| {
                assistant_message_part_table::get_by_harness_id(
                    conn,
                    assistant_message_id,
                    &harness_part_id,
                )
            })
            .await?)
    }

    pub async fn create_assistant_message_part(
        &self,
        part: AssistantMessagePart,
    ) -> Result<AssistantMessagePart, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_part_table::create(conn, &part))
            .await?)
    }

    pub async fn update_assistant_message_part(
        &self,
        part: AssistantMessagePart,
    ) -> Result<AssistantMessagePart, DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_part_table::update(conn, &part))
            .await?)
    }

    pub async fn delete_assistant_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError> {
        Ok(self
            .conn
            .call(move |conn| assistant_message_part_table::delete(conn, part_id))
            .await?)
    }
}
