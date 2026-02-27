use crate::backend::{
    Message, MessagePart, MessagePartAttachment, MessagePartFileSource, MessagePartPatchFile,
    MessageTool, Project, Session,
};

use super::{Database, DatabaseError, DatabaseStartupError, migrations::SQLITE_MIGRATIONS};
use tokio_rusqlite::{Connection as AsyncConnection, Error as AsyncError, rusqlite::Connection};
use uuid::Uuid;

mod message;
mod message_part;
mod project;
#[cfg(test)]
mod project_test;
mod session;
#[cfg(test)]
mod session_test;

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
        self.with_conn(project::list_projects).await
    }

    async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
        self.with_conn(move |conn| project::get_project(conn, project_id))
            .await
    }

    async fn create_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(move |conn| project::create_project(conn, &project))
            .await
    }

    async fn update_project(&self, project: Project) -> Result<Project, DatabaseError> {
        self.with_conn(move |conn| project::update_project(conn, &project))
            .await
    }

    async fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| project::delete_project(conn, project_id))
            .await
    }

    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, DatabaseError> {
        self.with_conn(move |conn| session::list_sessions_by_project(conn, project_id))
            .await
    }

    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
        self.with_conn(move |conn| session::get_session(conn, session_id))
            .await
    }

    async fn create_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(move |conn| session::create_session(conn, &session))
            .await
    }

    async fn update_session(&self, session: Session) -> Result<Session, DatabaseError> {
        self.with_conn(move |conn| session::update_session(conn, &session))
            .await
    }

    async fn delete_session(&self, session_id: uuid::Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| session::delete_session(conn, session_id))
            .await
    }

    async fn list_messages_by_session(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<Message>, DatabaseError> {
        self.with_conn(move |conn| message::list_messages_by_session(conn, session_id))
            .await
    }

    async fn get_message(&self, message_id: Uuid) -> Result<Option<Message>, DatabaseError> {
        self.with_conn(move |conn| message::get_message(conn, message_id))
            .await
    }

    async fn create_message(&self, message_item: Message) -> Result<Message, DatabaseError> {
        self.with_conn(move |conn| message::create_message(conn, &message_item))
            .await
    }

    async fn update_message(&self, message_item: Message) -> Result<Message, DatabaseError> {
        self.with_conn(move |conn| message::update_message(conn, &message_item))
            .await
    }

    async fn delete_message(&self, message_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| message::delete_message(conn, message_id))
            .await
    }

    async fn list_message_tools(&self, message_id: Uuid) -> Result<Vec<MessageTool>, DatabaseError> {
        self.with_conn(move |conn| message::list_message_tools(conn, message_id))
            .await
    }

    async fn upsert_message_tool(&self, tool: MessageTool) -> Result<MessageTool, DatabaseError> {
        self.with_conn(move |conn| message::upsert_message_tool(conn, &tool))
            .await
    }

    async fn delete_message_tool(
        &self,
        message_id: Uuid,
        tool_name: String,
    ) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| message::delete_message_tool(conn, message_id, &tool_name))
            .await
    }

    async fn list_message_parts_by_message(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<MessagePart>, DatabaseError> {
        self.with_conn(move |conn| message_part::list_parts_by_message(conn, message_id))
            .await
    }

    async fn get_message_part(&self, part_id: Uuid) -> Result<Option<MessagePart>, DatabaseError> {
        self.with_conn(move |conn| message_part::get_part(conn, part_id))
            .await
    }

    async fn create_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError> {
        self.with_conn(move |conn| message_part::create_part(conn, &part))
            .await
    }

    async fn update_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError> {
        self.with_conn(move |conn| message_part::update_part(conn, &part))
            .await
    }

    async fn delete_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| message_part::delete_part(conn, part_id))
            .await
    }

    async fn list_message_part_attachments(
        &self,
        part_id: Uuid,
    ) -> Result<Vec<MessagePartAttachment>, DatabaseError> {
        self.with_conn(move |conn| message_part::list_attachments_by_part(conn, part_id))
            .await
    }

    async fn create_message_part_attachment(
        &self,
        attachment: MessagePartAttachment,
    ) -> Result<MessagePartAttachment, DatabaseError> {
        self.with_conn(move |conn| message_part::create_attachment(conn, &attachment))
            .await
    }

    async fn delete_message_part_attachment(&self, attachment_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| message_part::delete_attachment(conn, attachment_id))
            .await
    }

    async fn get_message_part_file_source(
        &self,
        part_id: Uuid,
    ) -> Result<Option<MessagePartFileSource>, DatabaseError> {
        self.with_conn(move |conn| message_part::get_file_source(conn, part_id))
            .await
    }

    async fn upsert_message_part_file_source(
        &self,
        source: MessagePartFileSource,
    ) -> Result<MessagePartFileSource, DatabaseError> {
        self.with_conn(move |conn| message_part::upsert_file_source(conn, &source))
            .await
    }

    async fn delete_message_part_file_source(&self, part_id: Uuid) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| message_part::delete_file_source(conn, part_id))
            .await
    }

    async fn list_message_part_patch_files(
        &self,
        part_id: Uuid,
    ) -> Result<Vec<MessagePartPatchFile>, DatabaseError> {
        self.with_conn(move |conn| message_part::list_patch_files_by_part(conn, part_id))
            .await
    }

    async fn create_message_part_patch_file(
        &self,
        patch_file: MessagePartPatchFile,
    ) -> Result<MessagePartPatchFile, DatabaseError> {
        self.with_conn(move |conn| message_part::create_patch_file(conn, &patch_file))
            .await
    }

    async fn delete_message_part_patch_file(
        &self,
        part_id: Uuid,
        file_path: String,
    ) -> Result<(), DatabaseError> {
        self.with_conn(move |conn| message_part::delete_patch_file(conn, part_id, &file_path))
            .await
    }
}
