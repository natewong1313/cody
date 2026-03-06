use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend::{
    proto_project,
    proto_utils::{format_naive_datetime, parse_naive_datetime, parse_uuid},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModel {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<ProjectModel> for proto_project::ProjectModel {
    fn from(project: ProjectModel) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name,
            dir: project.dir,
            created_at: format_naive_datetime(project.created_at),
            updated_at: format_naive_datetime(project.updated_at),
        }
    }
}
impl TryFrom<proto_project::ProjectModel> for ProjectModel {
    type Error = tonic::Status;

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
