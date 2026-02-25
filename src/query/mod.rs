use egui::Ui;
use tonic::transport::Endpoint;
use uuid::Uuid;

use crate::{
    BACKEND_ADDR,
    query::{
        project::{ProjectState, Projects, ProjectsState},
        session::{Sessions, SessionsState},
    },
};

mod project;
mod session;

#[derive(Debug, Clone)]
pub enum QueryState<T> {
    Loading,
    Error(String),
    Data(T),
}

pub struct QueryClient {
    projects: Projects,
    sessions: Sessions,
}

impl QueryClient {
    pub fn new() -> Self {
        let backend_channel = Endpoint::from_shared(format!("http://{}", BACKEND_ADDR))
            .unwrap()
            .connect_lazy();
        let projects = Projects::new(backend_channel.clone());
        projects.listen_updates();
        let sessions = Sessions::new(backend_channel);

        Self { projects, sessions }
    }

    pub fn use_projects(&mut self, ui: &Ui) -> ProjectsState {
        self.projects.subscribe_state(ui)
    }

    pub fn use_project(&mut self, ui: &Ui, project_id: Uuid) -> ProjectState {
        self.projects.subscribe_project_state(ui, project_id)
    }

    pub fn use_sessions_by_project(&mut self, ui: &Ui, project_id: Uuid) -> SessionsState {
        self.sessions.subscribe_state(ui, project_id)
    }
}
