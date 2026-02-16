use super::{BackendServer, Project, query::query_all_projects};
use tarpc::context::Context;

#[tarpc::service]
pub trait Mutations {
    async fn create_project(project: Project) -> anyhow::Result<()>;
}

/// Define any mutations a user can call here
/// Every mutation should also call self.emit_* to update listeners
impl Mutations for BackendServer {
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
        let projects = query_all_projects(&db_conn)?;

        self.emit_project_update(project)?;
        self.emit_projects_update(projects)?;

        Ok(())
    }
}
