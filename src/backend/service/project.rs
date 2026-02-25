use std::{collections::hash_map::Entry, pin::Pin, sync::Arc};

use futures::{Stream, StreamExt, stream};
use tonic::{Request, Response, Status};

use super::required_field;
use crate::backend::{
    BackendService, Project,
    proto_project::{
        CreateProjectReply, CreateProjectRequest, DeleteProjectReply, DeleteProjectRequest,
        GetProjectReply, GetProjectRequest, ListProjectsReply, ListProjectsRequest,
        SubscribeProjectReply, SubscribeProjectRequest, SubscribeProjectsReply,
        SubscribeProjectsRequest, UpdateProjectReply, UpdateProjectRequest,
        project_server::Project as ProjectService,
    },
    proto_utils::parse_uuid,
};

#[tonic::async_trait]
impl ProjectService for Arc<BackendService> {
    type SubscribeProjectsStream =
        Pin<Box<dyn Stream<Item = Result<SubscribeProjectsReply, Status>> + Send + 'static>>;
    type SubscribeProjectStream =
        Pin<Box<dyn Stream<Item = Result<SubscribeProjectReply, Status>> + Send + 'static>>;

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

    async fn subscribe_project(
        &self,
        request: Request<SubscribeProjectRequest>,
    ) -> Result<Response<Self::SubscribeProjectStream>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        let project = self.project_repo.get(&project_id).await?;

        let receiver = {
            let mut senders = self
                .project_sender_by_id
                .lock()
                .map_err(|_| Status::internal("project sender lock poisoned"))?;

            match senders.entry(project_id) {
                Entry::Occupied(entry) => entry.get().subscribe(),
                Entry::Vacant(entry) => {
                    let (sender, receiver) = tokio::sync::watch::channel(project.clone());
                    entry.insert(sender);
                    receiver
                }
            }
        };

        let initial_reply = SubscribeProjectReply {
            project: project.map(Into::into),
        };
        let initial = stream::once(async move { Ok(initial_reply) });
        let updates = stream::unfold(receiver, |mut receiver| async move {
            if receiver.changed().await.is_err() {
                return None;
            }

            let reply = SubscribeProjectReply {
                project: receiver.borrow_and_update().clone().map(Into::into),
            };

            Some((Ok(reply), receiver))
        });

        Ok(Response::new(Box::pin(initial.chain(updates))))
    }

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectReply>, Status> {
        let req = request.into_inner();
        let model = required_field(req.project, "project")?;
        let project = Project::try_from(model)?;

        let created = self.project_repo.create(&project).await?;
        notify_project_subscribers(self, created.id, Some(created.clone()), "create")?;

        let current_projects = self.project_repo.list().await?;
        if self.projects_sender.send(current_projects).is_err() {
            log::debug!("No project subscribers to notify after create");
        }

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
        notify_project_subscribers(self, updated.id, Some(updated.clone()), "update")?;

        let current_projects = self.project_repo.list().await?;
        if self.projects_sender.send(current_projects).is_err() {
            log::debug!("No project subscribers to notify after update");
        }

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
        notify_project_subscribers(self, project_id, None, "delete")?;

        let current_projects = self.project_repo.list().await?;
        if self.projects_sender.send(current_projects).is_err() {
            log::debug!("No project subscribers to notify after delete");
        }

        Ok(Response::new(DeleteProjectReply {}))
    }

    async fn subscribe_projects(
        &self,
        _request: Request<SubscribeProjectsRequest>,
    ) -> Result<Response<Self::SubscribeProjectsStream>, Status> {
        let projects = self.project_repo.list().await?;
        let initial_reply = SubscribeProjectsReply {
            projects: projects.into_iter().map(Into::into).collect(),
        };
        let receiver = self.projects_sender.subscribe();
        let initial = stream::once(async move { Ok(initial_reply) });
        let updates = stream::unfold(receiver, |mut receiver| async move {
            if let Err(_) = receiver.changed().await {
                return None;
            }
            let reply = SubscribeProjectsReply {
                projects: receiver
                    .borrow_and_update()
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
            };
            Some((Ok(reply), receiver))
        });
        Ok(Response::new(Box::pin(initial.chain(updates))))
    }
}

fn notify_project_subscribers(
    backend: &BackendService,
    project_id: uuid::Uuid,
    project: Option<Project>,
    action: &str,
) -> Result<(), Status> {
    let sender = backend
        .project_sender_by_id
        .lock()
        .map_err(|_| Status::internal("project sender lock poisoned"))?
        .get(&project_id)
        .cloned();

    if let Some(sender) = sender
        && sender.send(project).is_err()
    {
        log::debug!("No project detail subscribers to notify after {action}");
    }

    Ok(())
}
