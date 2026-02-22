use thiserror::Error;
use uuid::Uuid;

use crate::backend::{Project, Session};

mod migrations;
pub mod sqlite;

#[derive(Error, Debug)]
pub enum DatabaseStartupError {
    #[error("Error establishing sqlite connection {0}")]
    SqliteConnection(#[from] rusqlite::Error),
    #[error("Error migrating sqlite {0}")]
    SqliteMigration(#[from] rusqlite_migration::Error),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Sqlite database error {0}")]
    SqliteQueryError(#[from] rusqlite::Error),
    #[error("Db conn lock poisoned")]
    PoisonedLock,
    #[error("{op} unexpected rows affected, expected {expected} got {actual}")]
    UnexpectedRowsAffected {
        op: &'static str,
        expected: usize,
        actual: usize,
    },
}

pub trait DatabaseTransaction {
    fn commit(&mut self) -> Result<(), DatabaseError>;
    fn rollback(&mut self) -> Result<(), DatabaseError>;
}

pub trait Database {
    type Transaction<'a>: DatabaseTransaction
    where
        Self: 'a;

    async fn begin_transaction(&self) -> Result<Self::Transaction<'_>, DatabaseError>;

    async fn list_projects(&self) -> Result<Vec<Project>, DatabaseError>;
    async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError>;
    async fn create_project(
        &self,
        project: Project,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Project, DatabaseError>;
    async fn update_project(
        &self,
        project: Project,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Project, DatabaseError>;
    async fn delete_project(
        &self,
        project_id: Uuid,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<(), DatabaseError>;

    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<Session>, DatabaseError>;
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError>;
    async fn create_session(
        &self,
        session: Session,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Session, DatabaseError>;
    async fn update_session(
        &self,
        session: Session,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<Session, DatabaseError>;
    async fn delete_session(
        &self,
        session_id: Uuid,
        tx: Option<&mut Self::Transaction<'_>>,
    ) -> Result<(), DatabaseError>;
}
