use super::{
    BackendServer, Project,
    query::{query_all_projects, query_project_by_id},
};
use tarpc::context::Context;
use uuid::Uuid;

#[tarpc::service]
pub trait BackendRpc {
    async fn list_projects() -> anyhow::Result<Vec<Project>>;
    async fn get_project(project_id: Uuid) -> anyhow::Result<Option<Project>>;
    async fn create_project(project: Project) -> anyhow::Result<Project>;
    async fn update_project(project: Project) -> anyhow::Result<Project>;
    async fn delete_project(project_id: Uuid) -> anyhow::Result<()>;
}

impl BackendRpc for BackendServer {
    async fn list_projects(self, _: Context) -> anyhow::Result<Vec<Project>> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;
        query_all_projects(&db_conn)
    }

    async fn get_project(self, _: Context, project_id: Uuid) -> anyhow::Result<Option<Project>> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;
        query_project_by_id(&db_conn, &project_id)
    }

    async fn create_project(self, _: Context, project: Project) -> anyhow::Result<Project> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;
        db_conn.execute(
            "INSERT INTO projects (id, name, dir) VALUES (?1, ?2, ?3)",
            (&project.id, &project.name, &project.dir),
        )?;
        Ok(project)
    }

    async fn update_project(self, _: Context, project: Project) -> anyhow::Result<Project> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;
        db_conn.execute(
            "UPDATE projects SET name = ?2, dir = ?3 WHERE id = ?1",
            (&project.id, &project.name, &project.dir),
        )?;
        Ok(project)
    }

    async fn delete_project(self, _: Context, project_id: Uuid) -> anyhow::Result<()> {
        let db_conn = self
            .db_conn
            .lock()
            .map_err(|e| anyhow::anyhow!("db_conn mutex poisoned {}", e))?;
        db_conn.execute("DELETE FROM projects WHERE id = ?1", [&project_id])?;
        Ok(())
    }
}
