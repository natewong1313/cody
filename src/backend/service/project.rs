use std::sync::Arc;
use tonic::{Request, Response, Status};

use super::required_field;
use crate::backend::{
    BackendService, Project,
    project::{
        CreateProjectReply, CreateProjectRequest, DeleteProjectReply, DeleteProjectRequest,
        GetProjectReply, GetProjectRequest, ListProjectsReply, ListProjectsRequest,
        SubscribeProjectsReply, SubscribeProjectsRequest, UpdateProjectReply, UpdateProjectRequest,
        project_server::Project as ProjectService,
    },
    proto_utils::parse_uuid,
};

#[tonic::async_trait]
impl ProjectService for Arc<BackendService> {
    async fn list_projects(
        &self,
        _request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsReply>, Status> {
        let projects = self.project_repo.list().await?;

        Ok(Response::new(ListProjectsReply {
            projects: projects.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectReply>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        let project = self.project_repo.get(&project_id).await?;

        Ok(Response::new(GetProjectReply {
            project: project.map(Into::into),
        }))
    }

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectReply>, Status> {
        let req = request.into_inner();
        let model = required_field(req.project, "project")?;
        let project = Project::try_from(model)?;

        let created = self.project_repo.create(&project).await?;

        Ok(Response::new(CreateProjectReply {
            project: Some(created.into()),
        }))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<UpdateProjectReply>, Status> {
        let req = request.into_inner();
        let model = required_field(req.project, "project")?;
        let project = Project::try_from(model)?;

        let updated = self.project_repo.update(&project).await?;

        Ok(Response::new(UpdateProjectReply {
            project: Some(updated.into()),
        }))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectReply>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        self.project_repo.delete(&project_id).await?;

        Ok(Response::new(DeleteProjectReply {}))
    }
}
