use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use tonic::{Request, transport::Channel};

use crate::backend::{Project, ProjectClient, SubscribeProjectsReply, SubscribeProjectsRequest};

use super::QueryState;

pub type ProjectsState = QueryState<Vec<Project>>;

pub struct Projects {
    backend_channel: Channel,
    state: ProjectsState,
    inbox: UiInbox<ProjectsState>,
}

impl Projects {
    pub fn new(backend_channel: Channel) -> Self {
        Self {
            backend_channel,
            state: QueryState::Loading,
            inbox: UiInbox::new(),
        }
    }

    pub fn listen_updates(&self) {
        let sender = self.inbox.sender().clone();
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
        if let Some(projects_update) = self.inbox.read(ui).last() {
            self.state = projects_update;
        }
        self.state.clone()
    }
}
