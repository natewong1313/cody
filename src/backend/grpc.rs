use std::{net::SocketAddr, sync::Arc};
use tokio::task::JoinHandle;
use tonic::{Request, Response, Status, transport::Server};

use crate::backend::{
    BackendContext, Project, Session,
    data::{
        project::{ProjectRepo, ProjectRepoError},
        session::{SessionRepo, SessionRepoError},
    },
    db::{DatabaseStartupError, sqlite::Sqlite},
    grpc::{project::project_server::ProjectServer, session::session_server::SessionServer},
    harness::{Harness, opencode::OpencodeHarness},
    proto_utils::parse_uuid,
};

pub mod project {
    tonic::include_proto!("project");
}
use project::project_server::Project as ProjectGrpc;
use project::{
    CreateProjectReply, CreateProjectRequest, DeleteProjectReply, DeleteProjectRequest,
    GetProjectReply, GetProjectRequest, ListProjectsReply, ListProjectsRequest, UpdateProjectReply,
    UpdateProjectRequest,
};

pub mod session {
    tonic::include_proto!("session");
}
use session::session_server::Session as SessionGrpc;
use session::{
    CreateSessionReply, CreateSessionRequest, DeleteSessionReply, DeleteSessionRequest,
    GetSessionReply, GetSessionRequest, ListSessionsByProjectReply, ListSessionsByProjectRequest,
    UpdateSessionReply, UpdateSessionRequest,
};

macro_rules! required_field {
    ($req:expr, $field:ident) => {
        $req.$field
            .ok_or_else(|| Status::invalid_argument(concat!("missing ", stringify!($field))))?
    };
}

pub struct BackendService {
    project_repo: ProjectRepo<Sqlite>,
    session_repo: SessionRepo<Sqlite>,
}

#[derive(thiserror::Error, Debug)]
pub enum BackendStartupError {
    #[error("Database initialization failed: {0}")]
    Database(#[from] DatabaseStartupError),
    #[error("Harness initialization failed: {0}")]
    Harness(String),
    #[error("Project repo initialization failed: {0}")]
    ProjectRepo(#[from] ProjectRepoError),
    #[error("Session repo initialization failed: {0}")]
    SessionRepo(#[from] SessionRepoError),
}

impl BackendService {
    pub fn new() -> Result<Self, BackendStartupError> {
        let db = Sqlite::new()?;
        let harness =
            OpencodeHarness::new().map_err(|e| BackendStartupError::Harness(e.to_string()))?;
        let ctx = BackendContext::new(db, harness);

        let project_repo = ProjectRepo::new(ctx.clone());
        let session_repo = SessionRepo::new(ctx.clone());

        Ok(Self {
            project_repo,
            session_repo,
        })
    }
}

#[tonic::async_trait]
impl ProjectGrpc for Arc<BackendService> {
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
        let model = required_field!(req, project);
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
        let model = required_field!(req, project);
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

#[tonic::async_trait]
impl SessionGrpc for Arc<BackendService> {
    async fn list_sessions_by_project(
        &self,
        request: Request<ListSessionsByProjectRequest>,
    ) -> Result<Response<ListSessionsByProjectReply>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        let sessions = self.session_repo.list_by_project(&project_id).await?;

        Ok(Response::new(ListSessionsByProjectReply {
            sessions: sessions.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        let session = self.session_repo.get(&session_id).await?;

        Ok(Response::new(GetSessionReply {
            session: session.map(Into::into),
        }))
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        let req = request.into_inner();
        let model = required_field!(req, session);
        let session = Session::try_from(model)?;

        let created = self.session_repo.create(&session).await?;

        Ok(Response::new(CreateSessionReply {
            session: Some(created.into()),
        }))
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<UpdateSessionReply>, Status> {
        let req = request.into_inner();
        let model = required_field!(req, session);
        let session = Session::try_from(model)?;

        let updated = self.session_repo.update(&session).await?;

        Ok(Response::new(UpdateSessionReply {
            session: Some(updated.into()),
        }))
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        self.session_repo.delete(&session_id).await?;

        Ok(Response::new(DeleteSessionReply {}))
    }
}

pub fn spawn_backend(
    addr: SocketAddr,
) -> Result<JoinHandle<Result<(), tonic::transport::Error>>, BackendStartupError> {
    let backend = Arc::new(BackendService::new()?);

    let project_service = ProjectServer::new(backend.clone());
    let session_service = SessionServer::new(backend);

    Ok(tokio::spawn(async move {
        log::info!("gRPC backend listening on {addr}");
        Server::builder()
            .add_service(project_service)
            .add_service(session_service)
            .serve(addr)
            .await
    }))
}
