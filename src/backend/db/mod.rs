use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    Message, MessagePart, MessagePartAttachment, MessagePartFileSource, MessagePartPatchFile,
    MessageTool, Project, Session,
    repo::assistant_message::{AssistantMessage, AssistantMessagePart},
    repo::user_message::{UserMessage, UserMessagePart},
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
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError>;
    async fn create_session(&self, session: Session) -> Result<Session, DatabaseError>;
    async fn update_session(&self, session: Session) -> Result<Session, DatabaseError>;
    async fn delete_session(&self, session_id: Uuid) -> Result<(), DatabaseError>;

    async fn list_messages_by_session(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<Message>, DatabaseError>;
    async fn get_message(&self, message_id: Uuid) -> Result<Option<Message>, DatabaseError>;
    async fn create_message(&self, message: Message) -> Result<Message, DatabaseError>;
    async fn update_message(&self, message: Message) -> Result<Message, DatabaseError>;
    async fn delete_message(&self, message_id: Uuid) -> Result<(), DatabaseError>;
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
    async fn create_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError>;
    async fn update_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError>;
    async fn delete_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError>;
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

    async fn get_user_message(
        &self,
        user_message_id: Uuid,
    ) -> Result<Option<UserMessage>, DatabaseError>;
    async fn create_user_message(
        &self,
        user_message: UserMessage,
    ) -> Result<UserMessage, DatabaseError>;
    async fn update_user_message(
        &self,
        user_message: UserMessage,
    ) -> Result<UserMessage, DatabaseError>;
    async fn delete_user_message(&self, user_message_id: Uuid) -> Result<(), DatabaseError>;

    async fn get_user_message_part(
        &self,
        part_id: Uuid,
    ) -> Result<Option<UserMessagePart>, DatabaseError>;
    async fn create_user_message_part(
        &self,
        part: UserMessagePart,
    ) -> Result<UserMessagePart, DatabaseError>;
    async fn update_user_message_part(
        &self,
        part: UserMessagePart,
    ) -> Result<UserMessagePart, DatabaseError>;
    async fn delete_user_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError>;

    async fn get_assistant_message(
        &self,
        assistant_message_id: Uuid,
    ) -> Result<Option<AssistantMessage>, DatabaseError>;
    async fn create_assistant_message(
        &self,
        assistant_message: AssistantMessage,
    ) -> Result<AssistantMessage, DatabaseError>;
    async fn update_assistant_message(
        &self,
        assistant_message: AssistantMessage,
    ) -> Result<AssistantMessage, DatabaseError>;
    async fn delete_assistant_message(
        &self,
        assistant_message_id: Uuid,
    ) -> Result<(), DatabaseError>;

    async fn get_assistant_message_part(
        &self,
        part_id: Uuid,
    ) -> Result<Option<AssistantMessagePart>, DatabaseError>;
    async fn create_assistant_message_part(
        &self,
        part: AssistantMessagePart,
    ) -> Result<AssistantMessagePart, DatabaseError>;
    async fn update_assistant_message_part(
        &self,
        part: AssistantMessagePart,
    ) -> Result<AssistantMessagePart, DatabaseError>;
    async fn delete_assistant_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError>;
}
