use std::{future::Future, sync::Arc};
use tonic::{Request, Response, Status};

use crate::backend::{
    BackendContext, BackendError, Project, Session,
    data::{
        project::{ProjectRepo, ProjectRepoError},
        session::{SessionRepo, SessionRepoError},
    },
    db::{DatabaseStartupError, sqlite::Sqlite},
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
    pub async fn new() -> Result<Self, BackendStartupError> {
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

    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[tonic::async_trait]
impl ProjectGrpc for Arc<BackendService> {
    async fn list_projects(
        &self,
        _request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsReply>, Status> {
        let projects = run_op_with(Arc::clone(self), |backend| async move {
            Ok(backend.project_repo.list().await?)
        })
        .await?;

        Ok(Response::new(ListProjectsReply {
            projects: projects.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectReply>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        let project = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.project_repo.get(&project_id).await?)
        })
        .await?;

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

        let created = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.project_repo.create(&project).await?)
        })
        .await?;

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

        let updated = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.project_repo.update(&project).await?)
        })
        .await?;

        Ok(Response::new(UpdateProjectReply {
            project: Some(updated.into()),
        }))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectReply>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        run_op_with(Arc::clone(self), move |backend| async move {
            backend.project_repo.delete(&project_id).await?;
            Ok(())
        })
        .await?;

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
        let sessions = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.session_repo.list_by_project(&project_id).await?)
        })
        .await?;

        Ok(Response::new(ListSessionsByProjectReply {
            sessions: sessions.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        let session = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.session_repo.get(&session_id).await?)
        })
        .await?
        .ok_or_else(|| Status::not_found(format!("session not found: {session_id}")))?;

        Ok(Response::new(GetSessionReply {
            session: Some(session.into()),
        }))
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        let req = request.into_inner();
        let model = required_field!(req, session);
        let session = Session::try_from(model)?;

        let created = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.session_repo.create(&session).await?)
        })
        .await?;

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

        let updated = run_op_with(Arc::clone(self), move |backend| async move {
            Ok(backend.session_repo.update(&session).await?)
        })
        .await?;

        Ok(Response::new(UpdateSessionReply {
            session: Some(updated.into()),
        }))
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        run_op_with(Arc::clone(self), move |backend| async move {
            backend.session_repo.delete(&session_id).await?;
            Ok(())
        })
        .await?;

        Ok(Response::new(DeleteSessionReply {}))
    }
}

fn map_backend_error(err: BackendError) -> Status {
    match err {
        BackendError::NotFound => Status::not_found("not found"),
        BackendError::ProjectNotFound(id) => Status::not_found(format!("project not found: {id}")),
        BackendError::Unavailable(message) => Status::unavailable(message),
        BackendError::Internal(message) => Status::internal(message),
    }
}

/// Helper functions for making sync db calls
async fn run_op_async<T, Fut, F>(f: F) -> Result<T, Status>
where
    T: Send + 'static,
    Fut: Future<Output = Result<T, BackendError>> + 'static,
    F: FnOnce() -> Fut + Send + 'static,
{
    let handle = tokio::runtime::Handle::current();
    let result = tokio::task::spawn_blocking(move || handle.block_on(f()))
        .await
        .map_err(|e| Status::internal(format!("backend task join error: {e}")))?;
    result.map_err(map_backend_error)
}
async fn run_op_with<T, Fut, F>(backend: Arc<BackendService>, f: F) -> Result<T, Status>
where
    T: Send + 'static,
    Fut: Future<Output = Result<T, BackendError>> + 'static,
    F: FnOnce(Arc<BackendService>) -> Fut + Send + 'static,
{
    run_op_async(move || f(backend)).await
}
