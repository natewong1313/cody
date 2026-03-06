use thiserror::Error;
use uuid::Uuid;

use crate::backend::{BackendContext, db::DatabaseError, models::project_model::ProjectModel};

#[derive(Debug, Error)]
pub enum ProjectRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
}

impl From<ProjectRepoError> for tonic::Status {
    fn from(err: ProjectRepoError) -> Self {
        match err {
            ProjectRepoError::Database(e) => tonic::Status::internal(e.to_string()),
        }
    }
}

pub struct ProjectRepo {
    ctx: BackendContext,
}

impl ProjectRepo {
    pub fn new(ctx: BackendContext) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<Vec<ProjectModel>, ProjectRepoError> {
        Ok(self.ctx.db.list_projects().await?)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<ProjectModel>, ProjectRepoError> {
        Ok(self.ctx.db.get_project(*id).await?)
    }

    pub async fn create(&self, project: &ProjectModel) -> Result<ProjectModel, ProjectRepoError> {
        Ok(self.ctx.db.create_project(project.clone()).await?)
    }

    pub async fn update(&self, project: &ProjectModel) -> Result<ProjectModel, ProjectRepoError> {
        Ok(self.ctx.db.update_project(project.clone()).await?)
    }

    pub async fn delete(&self, project_id: &Uuid) -> Result<(), ProjectRepoError> {
        self.ctx.db.delete_project(*project_id).await?;
        Ok(())
    }
}
