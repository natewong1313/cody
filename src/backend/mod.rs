use egui_inbox::{UiInbox, UiInboxSender};
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::sync::{Arc, Mutex};
use tarpc::context::Context;
use uuid::Uuid;

mod harness;
mod opencode_client;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
}
pub type ProjectsInbox = UiInbox<Vec<Project>>;
type ProjectsSender = UiInboxSender<Vec<Project>>;

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
    // TODO: if we eventually have a lot of data updates, we could build our own batching layer on
    // top of channels instead of inboxes so the gui doesn't repaint as much
    projects_sender: ProjectsSender,
}

impl BackendServer {
    pub fn new(projects_sender: ProjectsSender) -> Self {
        let mut db_conn = Connection::open_in_memory().unwrap();
        db_conn
            .pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))
            .unwrap();
        MIGRATIONS.to_latest(&mut db_conn).unwrap();

        Self {
            db_conn: Arc::new(Mutex::new(db_conn)),
            projects_sender,
        }
    }
}

impl BackendServer {
    fn emit_projects_update(&self, db_conn: &Connection) {
        let projects = self.query_all_projects(&db_conn).unwrap();
        self.projects_sender.send(projects).unwrap()
    }

    fn query_all_projects(&self, db_conn: &Connection) -> anyhow::Result<Vec<Project>> {
        let mut stmt = db_conn.prepare("SELECT id, name, dir FROM projects")?;
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

// todo: rename to mutations or something
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

        self.emit_projects_update(&db_conn);

        Ok(())
    }

    async fn get_projects(self, _: Context) -> anyhow::Result<Vec<Project>> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;

        self.query_all_projects(&db_conn)
    }
}
