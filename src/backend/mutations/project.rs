use tonic::{Request, transport::Channel};

use crate::backend::{Project, ProjectClient, proto_project::CreateProjectRequest};

pub fn create_project(backend_channel: Channel, project: Project) {
    tokio::spawn(async move {
        log::debug!("creating project");
        let mut client = ProjectClient::new(backend_channel);
        let request = CreateProjectRequest {
            project: Some(project.into()),
        };

        log::debug!("sending create project request");
        if let Err(error) = client.create_project(Request::new(request)).await {
            log::error!("failed to create project via gRPC: {error}");
        }
    });
}
