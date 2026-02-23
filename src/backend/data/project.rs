use chrono::NaiveDateTime;
use thiserror::Error;
use uuid::Uuid;

use crate::backend::{BackendContext, db::DatabaseError};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Error)]
pub enum ProjectRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
}

pub struct ProjectRepo<D>
where
    D: crate::backend::db::Database,
{
    ctx: BackendContext<D>,
}

impl<D> ProjectRepo<D>
where
    D: crate::backend::db::Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<Vec<Project>, ProjectRepoError> {
        Ok(self.ctx.db.list_projects().await?)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Project>, ProjectRepoError> {
        Ok(self.ctx.db.get_project(*id).await?)
    }

    pub async fn create(&self, project: &Project) -> Result<Project, ProjectRepoError> {
        Ok(self.ctx.db.create_project(project.clone(), None).await?)
    }

    pub async fn update(&self, project: &Project) -> Result<Project, ProjectRepoError> {
        Ok(self.ctx.db.update_project(project.clone(), None).await?)
    }

    pub async fn delete(&self, project_id: &Uuid) -> Result<(), ProjectRepoError> {
        self.ctx.db.delete_project(*project_id, None).await?;
        Ok(())
    }
}
