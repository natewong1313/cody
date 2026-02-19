use self::harness::{Harness, OpencodeHarness};
use crate::backend::db::Database;
use chrono::NaiveDateTime;
use rusqlite::Row;
use uuid::Uuid;

pub mod data;
mod db;
mod db_migrations;
mod harness;
mod opencode_client;
pub mod rpc;

#[derive(Clone)]
pub struct BackendServer {
    db: Database,
    harness: OpencodeHarness,
    sender: BackendEventSender,
}

#[derive(Debug, Clone)]
pub enum BackendEvent {
    // Projects(Vec<Project>),
    // Sessions(Vec<Session>),
    ProjectUpserted(Project),
    ProjectDeleted(Uuid),
    SessionUpserted(Session),
    SessionDeleted(Uuid),
}
type BackendEventSender = tokio::sync::broadcast::Sender<BackendEvent>;

/// The parent TARPC server, rpc routes are in rpc.rs
impl BackendServer {
    pub fn new(sender: BackendEventSender) -> Self {
        Self {
            db: Database::new().expect("failed to create database"),
            harness: OpencodeHarness::new().expect("failed to initialize opencode harness"),
            sender,
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BackendEvent> {
        self.sender.subscribe()
    }

    pub(super) fn emit_event(&self, event: BackendEvent) {
        if let Err(e) = self.sender.send(event) {
            eprintln!("error emitting event {:?}", e);
        };
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Project {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            dir: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub show_in_gui: bool,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Session {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            project_id: row.get(1)?,
            show_in_gui: row.get(2)?,
            name: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}
