use egui_inbox::{UiInbox, UiInboxSender};
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub mod mutations;

mod harness;
mod opencode_client;
mod query;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
}
pub type ProjectsInbox = UiInbox<Vec<Project>>;
type ProjectsSender = UiInboxSender<Vec<Project>>;

pub type ProjectInbox = UiInbox<(Uuid, Project)>;
type ProjectSender = UiInboxSender<(Uuid, Project)>;

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
    // TODO: if we eventually have a lot of data updates, we could build our own batching layer on
    // top of channels instead of inboxes so the gui doesn't repaint as much
    projects_sender: ProjectsSender,
    project_sender: ProjectSender,
}

impl BackendServer {
    pub fn new(projects_sender: ProjectsSender, project_sender: ProjectSender) -> Self {
        // Should be fine to call unwrap for now
        let mut db_conn = Connection::open_in_memory().unwrap();
        db_conn
            .pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))
            .unwrap();
        MIGRATIONS.to_latest(&mut db_conn).unwrap();

        Self {
            db_conn: Arc::new(Mutex::new(db_conn)),
            projects_sender,
            project_sender,
        }
    }

    fn emit_projects_update(&self, projects: Vec<Project>) -> anyhow::Result<()> {
        self.projects_sender
            .send(projects)
            .map_err(|e| anyhow::anyhow!("send projects update {:?}", e))?;
        Ok(())
    }

    fn emit_project_update(&self, project: Project) -> anyhow::Result<()> {
        self.project_sender
            .send((project.id, project))
            .map_err(|e| anyhow::anyhow!("send project update {:?}", e))?;
        Ok(())
    }
}
