use std::collections::{HashMap, HashSet};

use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use tonic::{Request, transport::Channel};
use uuid::Uuid;

use crate::backend::{
    Project, ProjectClient, SubscribeProjectReply, SubscribeProjectRequest, SubscribeProjectsReply,
    SubscribeProjectsRequest,
};

use super::QueryState;

pub type ProjectsState = QueryState<Vec<Project>>;
pub type ProjectState = QueryState<Option<Project>>;

pub struct Projects {
    backend_channel: Channel,
    projects_state: ProjectsState,
    projects_inbox: UiInbox<ProjectsState>,
    state_by_project: HashMap<Uuid, ProjectState>,
    project_subscriptions: HashSet<Uuid>,
    project_inbox: UiInbox<(Uuid, ProjectState)>,
}

impl Projects {
    pub fn new(backend_channel: Channel) -> Self {
        Self {
            backend_channel,
            projects_state: QueryState::Loading,
            projects_inbox: UiInbox::new(),
            state_by_project: HashMap::new(),
            project_subscriptions: HashSet::new(),
            project_inbox: UiInbox::new(),
        }
    }

    pub fn listen_updates(&self) {
        let sender = self.projects_inbox.sender().clone();
        let channel = self.backend_channel.clone();

        tokio::spawn(async move {
            let mut stream = match ProjectClient::new(channel)
                .subscribe_projects(Request::new(SubscribeProjectsRequest {}))
                .await
            {
                Ok(resp) => resp.into_inner(),
                Err(e) => {
                    let _ = sender.send(QueryState::Error(e.to_string()));
                    return;
                }
            };

            while let Some(next) = stream.next().await {
                match next.map_err(|e| e.to_string()).and_then(Projects::map) {
                    Ok(projects) => {
                        let _ = sender.send(QueryState::Data(projects));
                    }
                    Err(e) => {
                        let _ = sender.send(QueryState::Error(e.to_string()));
                    }
                };
            }

            let _ = sender.send(QueryState::Error(
                "projects stream closed unexpectedly".to_string(),
            ));
        });
    }

    fn map(reply: SubscribeProjectsReply) -> Result<Vec<Project>, String> {
        reply
            .projects
            .into_iter()
            .map(Project::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn subscribe_state(&mut self, ui: &Ui) -> ProjectsState {
        if let Some(projects_update) = self.projects_inbox.read(ui).last() {
            self.projects_state = projects_update;
        }
        self.projects_state.clone()
    }

    pub fn subscribe_project_state(&mut self, ui: &Ui, project_id: Uuid) -> ProjectState {
        for (updated_project_id, updated_state) in self.project_inbox.read(ui) {
            self.state_by_project
                .insert(updated_project_id, updated_state);
        }

        self.subscribe_project_if_needed(project_id);

        self.state_by_project
            .get(&project_id)
            .cloned()
            .unwrap_or(QueryState::Loading)
    }

    fn subscribe_project_if_needed(&mut self, project_id: Uuid) {
        if self.project_subscriptions.contains(&project_id) {
            return;
        }

        self.project_subscriptions.insert(project_id);
        self.state_by_project
            .insert(project_id, QueryState::Loading);

        let sender = self.project_inbox.sender().clone();
        let channel = self.backend_channel.clone();

        tokio::spawn(async move {
            let mut stream = match ProjectClient::new(channel)
                .subscribe_project(Request::new(SubscribeProjectRequest {
                    project_id: project_id.to_string(),
                }))
                .await
            {
                Ok(resp) => resp.into_inner(),
                Err(e) => {
                    let _ = sender.send((project_id, QueryState::Error(e.to_string())));
                    return;
                }
            };

            while let Some(next) = stream.next().await {
                match next
                    .map_err(|e| e.to_string())
                    .and_then(Projects::map_project)
                {
                    Ok(project) => {
                        let _ = sender.send((project_id, QueryState::Data(project)));
                    }
                    Err(e) => {
                        let _ = sender.send((project_id, QueryState::Error(e)));
                        return;
                    }
                }
            }

            let _ = sender.send((
                project_id,
                QueryState::Error("project stream closed unexpectedly".to_string()),
            ));
        });
    }

    fn map_project(reply: SubscribeProjectReply) -> Result<Option<Project>, String> {
        reply
            .project
            .map(Project::try_from)
            .transpose()
            .map_err(|e| e.to_string())
    }
}
