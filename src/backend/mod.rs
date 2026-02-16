use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "CREATE TABLE projects (
            id BLOB CHECK(length(id) = 16),
            name TEXT NOT NULL,
            dir TEXT NOT NULL
        );",
    ),
    // In the future, add more migrations here:
    //M::up("ALTER TABLE friend ADD COLUMN email TEXT;"),
];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

#[derive(Clone)]
pub struct BackendServer {
    db_conn: Arc<Mutex<Connection>>,
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
        }
    }
}
