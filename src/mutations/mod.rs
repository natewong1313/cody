use poll_promise::Promise;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::backend::{ProjectModel, SessionModel};

mod project;
mod session;

pub struct MutationsClient {
    backend_channel: Channel,
}

impl MutationsClient {
    pub fn new(backend_channel: Channel) -> Self {
        Self { backend_channel }
    }

    #[allow(dead_code)]
    pub fn create_session(&self, session: SessionModel) {
        session::create_session(self.backend_channel.clone(), session);
    }

    pub fn create_project_with_initial_session(
        &self,
        project: ProjectModel,
        session: SessionModel,
    ) -> Promise<Result<Uuid, String>> {
        project::create_project_with_initial_session(self.backend_channel.clone(), project, session)
    }
}
