use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    Project, Session,
    repo::message::{Message, MessagePart},
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
    async fn set_session_harness_id(
        &self,
        session_id: Uuid,
        harness_id: String,
    ) -> Result<(), DatabaseError>;
    async fn get_session_harness_id(
        &self,
        session_id: Uuid,
    ) -> Result<Option<String>, DatabaseError>;
    async fn get_session_id_by_harness_id(
        &self,
        harness_id: &str,
    ) -> Result<Option<Uuid>, DatabaseError>;

    async fn upsert_session_message(&self, message: Message) -> Result<(), DatabaseError>;
    async fn upsert_session_message_with_parts(
        &self,
        message: Message,
    ) -> Result<(), DatabaseError>;
    async fn ensure_session_message_exists(
        &self,
        session_id: Uuid,
        message_id: &str,
    ) -> Result<(), DatabaseError>;
    async fn mark_session_message_removed(
        &self,
        session_id: Uuid,
        message_id: &str,
    ) -> Result<(), DatabaseError>;
    async fn upsert_session_message_part(
        &self,
        session_id: Uuid,
        part: MessagePart,
        delta: Option<String>,
    ) -> Result<(), DatabaseError>;
    async fn list_session_messages(
        &self,
        session_id: Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<Message>, DatabaseError>;
}
