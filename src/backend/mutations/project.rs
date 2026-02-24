use tonic::{Request, transport::Channel};

use crate::backend::{
    Project, ProjectClient, Session, SessionClient, proto_project::CreateProjectRequest,
    proto_session::CreateSessionRequest,
};

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

pub fn create_project_with_initial_session(
    backend_channel: Channel,
    project: Project,
    session: Session,
) {
    tokio::spawn(async move {
        log::debug!("creating project and initial session");

        let mut project_client = ProjectClient::new(backend_channel.clone());
        let create_project_request = CreateProjectRequest {
            project: Some(project.into()),
        };

        if let Err(error) = project_client
            .create_project(Request::new(create_project_request))
            .await
        {
            log::error!("failed to create project via gRPC: {error}");
            return;
        }

        let mut session_client = SessionClient::new(backend_channel);
        let create_session_request = CreateSessionRequest {
            session: Some(session.into()),
        };

        if let Err(error) = session_client
            .create_session(Request::new(create_session_request))
            .await
        {
            log::error!("failed to create initial session via gRPC: {error}");
        }
    });
}
