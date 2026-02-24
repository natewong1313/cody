use egui::Ui;
use tonic::transport::Endpoint;

use crate::{
    BACKEND_ADDR,
    query::project::{Projects, ProjectsState},
};

mod project;

#[derive(Debug, Clone)]
pub enum QueryState<T> {
    Loading,
    Error(String),
    Data(T),
}

pub struct QueryClient {
    projects: Projects,
}

impl QueryClient {
    pub fn new() -> Self {
        let backend_channel = Endpoint::from_shared(format!("http://{}", BACKEND_ADDR))
            .unwrap()
            .connect_lazy();
        let projects = Projects::new(backend_channel.clone());
        projects.listen_updates();

        Self { projects }
    }

    pub fn use_projects(&mut self, ui: &Ui) -> ProjectsState {
        self.projects.subscribe_state(ui)
    }
}
