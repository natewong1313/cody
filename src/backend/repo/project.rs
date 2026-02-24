use chrono::NaiveDateTime;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::DatabaseError,
    proto_project,
    proto_utils::{format_naive_datetime, parse_naive_datetime, parse_uuid},
};

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

impl From<ProjectRepoError> for tonic::Status {
    fn from(err: ProjectRepoError) -> Self {
        match err {
            ProjectRepoError::Database(e) => tonic::Status::internal(e.to_string()),
        }
    }
}

impl From<Project> for proto_project::ProjectModel {
    fn from(project: Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            dir: project.dir,
            created_at: format_naive_datetime(project.created_at),
            updated_at: format_naive_datetime(project.updated_at),
        }
    }
}

impl TryFrom<proto_project::ProjectModel> for Project {
    type Error = Status;

    fn try_from(model: proto_project::ProjectModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: parse_uuid("project.id", &model.id)?,
            name: model.name,
            dir: model.dir,
            created_at: parse_naive_datetime("project.created_at", &model.created_at)?,
            updated_at: parse_naive_datetime("project.updated_at", &model.updated_at)?,
        })
    }
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
        Ok(self.ctx.db.create_project(project.clone()).await?)
    }

    pub async fn update(&self, project: &Project) -> Result<Project, ProjectRepoError> {
        Ok(self.ctx.db.update_project(project.clone()).await?)
    }

    pub async fn delete(&self, project_id: &Uuid) -> Result<(), ProjectRepoError> {
        self.ctx.db.delete_project(*project_id).await?;
        Ok(())
    }
}
