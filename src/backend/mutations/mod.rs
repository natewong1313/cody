use tonic::transport::Channel;

use crate::backend::{Project, Session};

mod project;
mod session;

pub struct MutationsClient {
    backend_channel: Channel,
}

impl MutationsClient {
    pub fn new(backend_channel: Channel) -> Self {
        Self { backend_channel }
    }

    pub fn create_project(&self, project: Project) {
        project::create_project(self.backend_channel.clone(), project);
    }

    pub fn create_session(&self, session: Session) {
        session::create_session(self.backend_channel.clone(), session);
    }

    pub fn create_project_with_initial_session(&self, project: Project, session: Session) {
        project::create_project_with_initial_session(
            self.backend_channel.clone(),
            project,
            session,
        );
    }
}
