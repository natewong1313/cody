use std::sync::Arc;

use crate::backend::{LocalBackend, LocalBackendStartupError};

use super::{Backend, BackendError, Project, Session};
use futures::StreamExt;
use uuid::Uuid;

#[tarpc::service]
pub trait BackendRpc {
    async fn list_projects() -> Result<Vec<Project>, BackendError>;
    async fn get_project(project_id: Uuid) -> Result<Option<Project>, BackendError>;
    async fn create_project(project: Project) -> Result<Project, BackendError>;
    async fn update_project(project: Project) -> Result<Project, BackendError>;
    async fn delete_project(project_id: Uuid) -> Result<(), BackendError>;
    async fn list_sessions_by_project(project_id: Uuid) -> Result<Vec<Session>, BackendError>;
    async fn get_session(session_id: Uuid) -> Result<Option<Session>, BackendError>;
    async fn create_session(session: Session) -> Result<Session, BackendError>;
    async fn update_session(session: Session) -> Result<Session, BackendError>;
    async fn delete_session(session_id: Uuid) -> Result<(), BackendError>;
}

struct BackendRpcServer<B>
where
    B: Backend,
{
    backend: Arc<B>,
}

impl<B> Clone for BackendRpcServer<B>
where
    B: Backend,
{
    fn clone(&self) -> Self {
        Self {
            backend: Arc::clone(&self.backend),
        }
    }
}

impl<B> BackendRpc for BackendRpcServer<B>
where
    B: Backend,
{
    async fn list_projects(self, _: tarpc::context::Context) -> Result<Vec<Project>, BackendError> {
        self.backend.list_projects().await
    }

    async fn get_project(
        self,
        _: tarpc::context::Context,
        project_id: Uuid,
    ) -> Result<Option<Project>, BackendError> {
        self.backend.get_project(project_id).await
    }

    async fn create_project(
        self,
        _: tarpc::context::Context,
        project: Project,
    ) -> Result<Project, BackendError> {
        self.backend.create_project(project).await
    }

    async fn update_project(
        self,
        _: tarpc::context::Context,
        project: Project,
    ) -> Result<Project, BackendError> {
        self.backend.update_project(project).await
    }

    async fn delete_project(
        self,
        _: tarpc::context::Context,
        project_id: Uuid,
    ) -> Result<(), BackendError> {
        self.backend.delete_project(project_id).await
    }

    async fn list_sessions_by_project(
        self,
        _: tarpc::context::Context,
        project_id: Uuid,
    ) -> Result<Vec<Session>, BackendError> {
        self.backend.list_sessions_by_project(project_id).await
    }

    async fn get_session(
        self,
        _: tarpc::context::Context,
        session_id: Uuid,
    ) -> Result<Option<Session>, BackendError> {
        self.backend.get_session(session_id).await
    }

    async fn create_session(
        self,
        _: tarpc::context::Context,
        session: Session,
    ) -> Result<Session, BackendError> {
        self.backend.create_session(session).await
    }

    async fn update_session(
        self,
        _: tarpc::context::Context,
        session: Session,
    ) -> Result<Session, BackendError> {
        self.backend.update_session(session).await
    }

    async fn delete_session(
        self,
        _: tarpc::context::Context,
        session_id: Uuid,
    ) -> Result<(), BackendError> {
        self.backend.delete_session(session_id).await
    }
}

pub async fn start_local_backend_rpc() -> Result<BackendRpcClient, LocalBackendStartupError> {
    use crate::backend::rpc::BackendRpc;
    use tarpc::client;

    let backend = LocalBackend::new().await?;
    let server_impl = BackendRpcServer {
        backend: Arc::new(backend),
    };
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to start backend rpc runtime");

        runtime.block_on(async move {
            use tarpc::server::{self, Channel};

            server::BaseChannel::with_defaults(server_transport)
                .execute(server_impl.serve())
                .for_each(|response| response)
                .await;
        });
    });

    Ok(BackendRpcClient::new(client::Config::default(), client_transport).spawn())
}
