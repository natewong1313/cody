use poll_promise::Promise;
use tonic::{Request, transport::Channel};
use uuid::Uuid;

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
) -> Promise<Result<Uuid, String>> {
    let project_id = project.id;

    Promise::spawn_async(async move {
        log::debug!("creating project and initial session");

        let mut project_client = ProjectClient::new(backend_channel.clone());
        let create_project_request = CreateProjectRequest {
            project: Some(project.into()),
        };

        project_client
            .create_project(Request::new(create_project_request))
            .await
            .map_err(|error| format!("failed to create project via gRPC: {error}"))?;

        let mut session_client = SessionClient::new(backend_channel);
        let create_session_request = CreateSessionRequest {
            session: Some(session.into()),
        };

        session_client
            .create_session(Request::new(create_session_request))
            .await
            .map_err(|error| format!("failed to create initial session via gRPC: {error}"))?;

        Ok(project_id)
    })
}
