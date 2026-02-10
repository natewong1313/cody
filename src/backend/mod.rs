use std::sync::{Arc, Mutex};

use futures::prelude::*;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use tarpc::{
    client,
    context::Context,
    server::{self, Channel},
};
use uuid::Uuid;

mod harness;
mod opencode_client;

#[derive(Debug)]
pub struct Project {
    id: Uuid,
    name: String,
    dir: String,
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

//
#[derive(Clone)]
pub struct BackendServer {
    db_conn: Arc<Mutex<Connection>>,
}

impl BackendServer {
    pub fn new() -> Self {
        // TODO: i think its fine to panic here since it makes the api less hell, but should re explore this
        // TODO: dont use in memory, figure out where the path should live (somewhere in
        // appdata/user data land)
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

#[tarpc::service]
pub trait Contract {
    async fn create_project(project: Project) -> anyhow::Result<()>;
    async fn get_projects() -> anyhow::Result<Vec<Project>>;
}

impl Contract for BackendServer {
    async fn create_project(self, _: Context, project: Project) -> anyhow::Result<()> {
        let db_conn = self
            .db_conn
            .lock()
            // TODO We might be cooked here, should add resiliancy
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;
        db_conn.execute(
            "INSERT INTO projects (id, name, dir) VALUES (?1, ?2, ?3)",
            (&project.id, &project.name, &project.dir),
        )?;
        Ok(())
    }

    async fn get_projects(self, _: Context) -> anyhow::Result<Vec<Project>> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;

        let mut stmt = db_conn.prepare("SELECT * FROM projects")?;
        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    dir: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }
}
