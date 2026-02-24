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

pub trait Database {
    fn list_projects(&self) -> Result<Vec<Project>, DatabaseError>;
    fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, DatabaseError>;
    fn create_project(&self, project: Project) -> Result<Project, DatabaseError>;
    fn update_project(&self, project: Project) -> Result<Project, DatabaseError>;
    fn delete_project(&self, project_id: Uuid) -> Result<(), DatabaseError>;

    fn list_sessions_by_project(&self, project_id: Uuid) -> Result<Vec<Session>, DatabaseError>;
    fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, DatabaseError>;
    fn create_session(&self, session: Session) -> Result<Session, DatabaseError>;
    fn update_session(&self, session: Session) -> Result<Session, DatabaseError>;
    fn delete_session(&self, session_id: Uuid) -> Result<(), DatabaseError>;
}
