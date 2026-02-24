use tonic::transport::Channel;

use crate::backend::Project;

mod project;

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
}
