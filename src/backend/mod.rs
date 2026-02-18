use self::harness::{Harness, OpencodeHarness};
use crate::backend::db::Database;
use chrono::NaiveDateTime;
use rusqlite::Row;
use uuid::Uuid;

mod db;
mod db_migrations;
mod harness;
mod opencode_client;
pub mod rpc;

#[derive(Clone)]
pub struct BackendServer {
    db: Database,
    harness: OpencodeHarness,
}

/// The parent TARPC server, rpc routes are in rpc.rs
impl BackendServer {
    pub fn new() -> Self {
        Self {
            db: Database::new().expect("failed to create database"),
            harness: OpencodeHarness::new().expect("failed to initialize opencode harness"),
        }
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
