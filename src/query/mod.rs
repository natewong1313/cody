use egui::{Context, Ui};
use egui_inbox::UiInbox;
use futures::StreamExt;
use tonic::{
    Request,
    transport::{Channel, Endpoint},
};

use crate::{
    BACKEND_ADDR,
    backend::{
        Project,
        project::{SubscribeProjectsRequest, project_client::ProjectClient},
    },
};

pub enum QueryUIMessage {
    ProjectsLoaded(Result<Vec<Project>, String>),
    ProjectCreated(Result<Project, String>),
}

pub struct QueryClient {
    backend_channel: Channel,
    updates_inbox: UiInbox<QueryUIMessage>,
    egui_ctx: Option<egui::Context>,
    projects: Vec<Project>,
    projects_status: Option<ProjectsStatus>,
}

pub enum ProjectsStatus {
    Pending,
    Error,
    Data(Vec<Project>),
}

impl QueryClient {
    pub fn new() -> Self {
        let backend_channel = Endpoint::from_shared(format!("http://{}", BACKEND_ADDR))
            .unwrap()
            .connect_lazy();
        QueryClient::spawn_projects_listener(&backend_channel);
        Self {
            backend_channel,
            updates_inbox: UiInbox::new(),
            egui_ctx: None,
            projects: Vec::new(),
            projects_status: None,
        }
    }

    pub fn connect(&mut self, ctx: &egui::Context) {
        self.egui_ctx = Some(ctx.clone());
    }

    fn spawn_projects_listener(backend_channel: &Channel) {
        let channel = backend_channel.clone();
        tokio::spawn(async move {
            let stream_result = ProjectClient::new(channel)
                .subscribe_projects(Request::new(SubscribeProjectsRequest {}))
                .await
                .map(|resp| resp.into_inner());
            match stream_result {
                Ok(mut stream) => {
                    while let Some(next) = stream.next().await {
                        let mapped = next.map_err(|e| e.to_string()).and_then(|reply| {
                            reply
                                .projects
                                .into_iter()
                                .map(Project::try_from)
                                .collect::<Result<Vec<_>, _>>()
                                .map_err(|e| e.to_string())
                        });
                        let _ = "";
                        // let _ = sender.send(QueryUIMessage::ProjectsLoaded(mapped));
                    }
                }
                Err(e) => {
                    let _ = "";
                    // let _ = sender.send(QueryUIMessage::ProjectsLoaded(Err(e.to_string())));
                }
            }
        });
    }

    pub fn use_projects(&mut self, ui: &mut Ui) -> ProjectsStatus {
        ProjectsStatus::Pending
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
