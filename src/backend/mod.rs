use rusqlite::Row;
use uuid::Uuid;

use crate::backend::db::Database;

use self::harness::{Harness, OpencodeHarness};

mod db;
mod harness;
mod opencode_client;
pub mod rpc;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
}

impl Project {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            dir: row.get(2)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
}

impl Session {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
        })
    }
}

#[derive(Clone)]
pub struct BackendServer {
    db: Database,
    harness: OpencodeHarness,
}

impl BackendServer {
    pub fn new() -> Self {
        Self {
            db: Database::new().expect("failed to create database"),
            harness: OpencodeHarness::new().expect("failed to initialize opencode harness"),
        }
    }
}
