use egui::Ui;
use tonic::transport::Endpoint;
use uuid::Uuid;

use crate::{
    BACKEND_ADDR,
    query::{
        message::{Messages, MessagesState},
        project::{ProjectState, Projects, ProjectsState},
        session::{Sessions, SessionsState},
    },
};

mod message;
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
    messages: Messages,
}

impl QueryClient {
    pub fn new() -> Self {
        let backend_channel = Endpoint::from_shared(format!("http://{}", BACKEND_ADDR))
            .unwrap()
            .connect_lazy();
        let projects = Projects::new(backend_channel.clone());
        projects.listen_updates();
        let sessions = Sessions::new(backend_channel.clone());
        let messages = Messages::new(backend_channel);

        Self {
            projects,
            sessions,
            messages,
        }
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

    pub fn use_messages_by_session(&mut self, ui: &Ui, session_id: Uuid) -> MessagesState {
        self.messages.subscribe_state(ui, session_id)
    }
}
