use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    Message, MessagePart, MessagePartAttachment, MessagePartFileSource, MessagePartPatchFile,
    MessageTool, Project, Session,
};

mod migrations;
pub mod sqlite;

#[derive(Error, Debug)]
pub enum DatabaseStartupError {
    #[error("Error establishing sqlite connection {0}")]
    SqliteConnection(#[from] tokio_rusqlite::rusqlite::Error),
    #[error("Error migrating sqlite {0}")]
    SqliteMigration(#[from] rusqlite_migration::Error),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Sqlite database error {0}")]
    SqliteQueryError(#[from] tokio_rusqlite::rusqlite::Error),
    #[error("Db connection closed")]
    ConnectionClosed,
    #[error("{op} unexpected rows affected, expected {expected} got {actual}")]
    UnexpectedRowsAffected {
        op: &'static str,
        expected: usize,
        actual: usize,
    },
}

pub trait Database {
    async fn list_projects(&self) -> Result<Vec<Project>, DatabaseError>;
    async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError>;
    async fn create_project(&self, project: Project) -> Result<Project, DatabaseError>;
    async fn update_project(&self, project: Project) -> Result<Project, DatabaseError>;
    async fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError>;

    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, DatabaseError>;
    async fn get_session_by_harness_session_id(
        &self,
        harness_session_id: String,
    ) -> Result<Option<Session>, DatabaseError>;
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError>;
    async fn create_session(&self, session: Session) -> Result<Session, DatabaseError>;
    async fn update_session(&self, session: Session) -> Result<Session, DatabaseError>;
    async fn delete_session(&self, session_id: Uuid) -> Result<(), DatabaseError>;

    async fn list_messages_by_session(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<Message>, DatabaseError>;
    async fn get_message(&self, message_id: Uuid) -> Result<Option<Message>, DatabaseError>;
    async fn get_message_by_harness_message_id(
        &self,
        session_id: Uuid,
        harness_message_id: String,
    ) -> Result<Option<Message>, DatabaseError>;
    async fn create_message(&self, message: Message) -> Result<Message, DatabaseError>;
    async fn update_message(&self, message: Message) -> Result<Message, DatabaseError>;
    async fn delete_message(&self, message_id: Uuid) -> Result<(), DatabaseError>;
    async fn delete_message_by_harness_message_id(
        &self,
        session_id: Uuid,
        harness_message_id: String,
    ) -> Result<(), DatabaseError>;
    async fn mark_session_assistant_messages_finished(
        &self,
        session_id: Uuid,
        completed_at: chrono::NaiveDateTime,
    ) -> Result<(), DatabaseError>;
    async fn list_message_tools(&self, message_id: Uuid)
    -> Result<Vec<MessageTool>, DatabaseError>;
    async fn upsert_message_tool(&self, tool: MessageTool) -> Result<MessageTool, DatabaseError>;
    async fn delete_message_tool(
        &self,
        message_id: Uuid,
        tool_name: String,
    ) -> Result<(), DatabaseError>;

    async fn list_message_parts_by_message(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<MessagePart>, DatabaseError>;
    async fn get_message_part(&self, part_id: Uuid) -> Result<Option<MessagePart>, DatabaseError>;
    async fn get_message_part_by_harness_part_id(
        &self,
        message_id: Uuid,
        harness_part_id: String,
    ) -> Result<Option<MessagePart>, DatabaseError>;
    async fn create_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError>;
    async fn update_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError>;
    async fn delete_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError>;
    async fn append_message_part_text_delta(
        &self,
        part_id: Uuid,
        delta: String,
    ) -> Result<MessagePart, DatabaseError>;
    async fn list_message_part_attachments(
        &self,
        part_id: Uuid,
    ) -> Result<Vec<MessagePartAttachment>, DatabaseError>;
    async fn create_message_part_attachment(
        &self,
        attachment: MessagePartAttachment,
    ) -> Result<MessagePartAttachment, DatabaseError>;
    async fn delete_message_part_attachment(
        &self,
        attachment_id: Uuid,
    ) -> Result<(), DatabaseError>;
    async fn get_message_part_file_source(
        &self,
        part_id: Uuid,
    ) -> Result<Option<MessagePartFileSource>, DatabaseError>;
    async fn upsert_message_part_file_source(
        &self,
        source: MessagePartFileSource,
    ) -> Result<MessagePartFileSource, DatabaseError>;
    async fn delete_message_part_file_source(&self, part_id: Uuid) -> Result<(), DatabaseError>;
    async fn list_message_part_patch_files(
        &self,
        part_id: Uuid,
    ) -> Result<Vec<MessagePartPatchFile>, DatabaseError>;
    async fn create_message_part_patch_file(
        &self,
        patch_file: MessagePartPatchFile,
    ) -> Result<MessagePartPatchFile, DatabaseError>;
    async fn delete_message_part_patch_file(
        &self,
        part_id: Uuid,
        file_path: String,
    ) -> Result<(), DatabaseError>;
}
