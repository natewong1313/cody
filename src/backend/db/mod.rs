use thiserror::Error;
use uuid::Uuid;

use crate::backend::{Message, MessagePart, Project, Session};

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

    async fn list_message_parts_by_message(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<MessagePart>, DatabaseError>;
    async fn get_message_part(&self, part_id: Uuid) -> Result<Option<MessagePart>, DatabaseError>;
    async fn create_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError>;
    async fn update_message_part(&self, part: MessagePart) -> Result<MessagePart, DatabaseError>;
    async fn delete_message_part(&self, part_id: Uuid) -> Result<(), DatabaseError>;
}
