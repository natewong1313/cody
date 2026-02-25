use poll_promise::Promise;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::backend::{Project, Session};

mod message;
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
    pub fn create_project(&self, project: Project) {
        project::create_project(self.backend_channel.clone(), project);
    }

    #[allow(dead_code)]
    pub fn create_session(&self, session: Session) {
        session::create_session(self.backend_channel.clone(), session);
    }

    pub fn send_message(&self, session_id: Uuid, text: String) {
        message::send_message(self.backend_channel.clone(), session_id, text);
    }

    pub fn create_project_with_initial_session(
        &self,
        project: Project,
        session: Session,
    ) -> Promise<Result<Uuid, String>> {
        project::create_project_with_initial_session(self.backend_channel.clone(), project, session)
    }
}
