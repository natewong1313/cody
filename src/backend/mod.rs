use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use self::harness::{Harness, OpencodeHarness};

pub mod rpc;

mod harness;
mod opencode_client;
mod query;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
}

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "CREATE TABLE projects (
            id BLOB CHECK(length(id) = 16),
            name TEXT NOT NULL,
            dir TEXT NOT NULL
        );",
    ),
    M::up(
        "CREATE TABLE sessions (
            id BLOB CHECK(length(id) = 16),
            project_id BLOB CHECK(length(project_id) = 16) REFERENCES projects(id),
            name TEXT NOT NULL
        );",
    ),
];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

#[derive(Clone)]
pub struct BackendServer {
    db_conn: Arc<Mutex<Connection>>,
    harness: OpencodeHarness,
}

impl BackendServer {
    pub fn new() -> Self {
        // Should be fine to call unwrap for now
        let mut db_conn = Connection::open_in_memory().unwrap();
        db_conn
            .pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))
            .unwrap();
        MIGRATIONS.to_latest(&mut db_conn).unwrap();

        Self {
            db_conn: Arc::new(Mutex::new(db_conn)),
            harness: OpencodeHarness::new().expect("failed to initialize opencode harness"),
        }
    }
}
