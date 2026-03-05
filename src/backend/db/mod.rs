use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    Project, Session,
    repo::assistant_message::{AssistantMessage, AssistantMessagePart},
    repo::message::Message,
    repo::user_message::UserMessage,
    repo::user_message_part::UserMessagePart,
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
    #[error("Serde rusqlite error {0}")]
    SerdeError(#[from] serde_rusqlite::Error),
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
        limit: u32,
    ) -> Result<Vec<Message>, DatabaseError>;

    async fn list_user_messages_by_session(
        &self,
        session_id: Uuid,
        limit: u32,
    ) -> Result<Vec<UserMessage>, DatabaseError>;
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
    async fn get_assistant_message_by_harness_id(
        &self,
        session_id: Uuid,
        harness_message_id: String,
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
    async fn get_assistant_message_part_by_harness_id(
        &self,
        assistant_message_id: Uuid,
        harness_part_id: String,
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
