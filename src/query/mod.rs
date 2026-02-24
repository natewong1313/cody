use egui::Context;
use egui_inbox::UiInbox;

use crate::backend::{Project, rpc::BackendRpcClient};

pub enum QueryUIMessage {
    ProjectsLoaded(Result<Vec<Project>, String>),
    ProjectCreated(Result<Project, String>),
}

pub struct QueryClient {
    backend: BackendRpcClient,
    updates_inbox: UiInbox<QueryUIMessage>,
    projects: Vec<Project>,
    projects_loading: bool,
    projects_error: Option<String>,
    projects_in_flight: bool,
}

impl QueryClient {
    pub fn new(backend: BackendRpcClient) -> Self {
        Self {
            backend,
            updates_inbox: UiInbox::new(),
            projects: Vec::new(),
            projects_loading: false,
            projects_error: None,
            projects_in_flight: false,
        }
    }
    //
    // pub fn poll(&mut self, ctx: &Context) {
    //     for msg in self.updates_inbox.read(ctx) {
    //         match msg {
    //             QueryUIMessage::ProjectsLoaded(result) => {
    //                 self.projects_in_flight = false;
    //                 self.projects_loading = false;
    //
    //                 match result {
    //                     Ok(projects) => {
    //                         self.projects = projects;
    //                         self.projects_error = None;
    //                     }
    //                     Err(message) => {
    //                         self.projects_error = Some(message);
    //                     }
    //                 }
    //             }
    //             QueryUIMessage::ProjectCreated(result) => match result {
    //                 Ok(project) => {
    //                     self.projects.retain(|p| p.id != project.id);
    //                     self.projects.push(project);
    //                     self.projects
    //                         .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    //                     self.projects_error = None;
    //                 }
    //                 Err(message) => {
    //                     self.projects_error = Some(message);
    //                 }
    //             },
    //         }
    //     }
    // }
    //
    // pub fn load_projects_if_needed(&mut self) {
    //     if self.projects_in_flight || self.projects_loading || !self.projects.is_empty() {
    //         return;
    //     }
    //
    //     self.projects_in_flight = true;
    //     self.projects_loading = true;
    //     self.projects_error = None;
    //
    //     let client = self.backend.clone();
    //     let sender = self.updates_inbox.sender();
    //     tokio::spawn(async move {
    //         let result = client
    //             .list_projects(context::current())
    //             .await
    //             .map_err(|e| e.to_string())
    //             .and_then(|r| r.map_err(|e| e.to_string()));
    //
    //         let _ = sender.send(QueryUIMessage::ProjectsLoaded(result));
    //     });
    // }
    //
    // pub fn projects(&self) -> &[Project] {
    //     &self.projects
    // }
    //
    // pub fn projects_loading(&self) -> bool {
    //     self.projects_loading
    // }
    //
    // pub fn projects_error(&self) -> Option<&str> {
    //     self.projects_error.as_deref()
    // }
    //
    // pub fn refresh_projects(&mut self) {
    //     self.projects.clear();
    //     self.projects_loading = false;
    //     self.projects_in_flight = false;
    // }
    //
    // pub fn create_project(&mut self, project: Project) {
    //     let client = self.backend.clone();
    //     let sender = self.updates_inbox.sender();
    //
    //     tokio::spawn(async move {
    //         let result = client
    //             .create_project(context::current(), project)
    //             .await
    //             .map_err(|e| e.to_string())
    //             .and_then(|r| r.map_err(|e| e.to_string()));
    //
    //         let _ = sender.send(QueryUIMessage::ProjectCreated(result));
    //     });
    // }
}
