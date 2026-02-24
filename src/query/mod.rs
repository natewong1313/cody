use std::sync::{Arc, RwLock};

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
        project::{
            SubscribeProjectsReply, SubscribeProjectsRequest, project_client::ProjectClient,
        },
    },
};

#[derive(Debug, Clone)]
pub enum QueryState<T> {
    Loading,
    Error(String),
    Data(T),
}

type Projects = Vec<Project>;

pub struct QueryClient {
    egui_ctx: Arc<RwLock<Option<Context>>>,
    // projects_state: Arc<RwLock<QueryState<Projects>>>,
    projects_state: QueryState<Projects>,
    projects_state_inbox: UiInbox<QueryState<Projects>>,
}

impl QueryClient {
    pub fn new() -> Self {
        let backend_channel = Endpoint::from_shared(format!("http://{}", BACKEND_ADDR))
            .unwrap()
            .connect_lazy();
        let egui_ctx = Arc::new(RwLock::new(None));

        let projects_state_inbox = UiInbox::new();
        QueryClient::spawn_projects_listener(backend_channel, &projects_state_inbox);

        Self {
            egui_ctx,
            projects_state: QueryState::Loading,
            projects_state_inbox,
        }
    }

    pub fn connect(&self, ctx: &Context) {
        *self.egui_ctx.write().expect("egui_ctx lock poisoned") = Some(ctx.clone());
    }

    fn spawn_projects_listener(backend_channel: Channel, inbox: &UiInbox<QueryState<Projects>>) {
        let sender = inbox.sender().clone();
        tokio::spawn(async move {
            let mut stream = match ProjectClient::new(backend_channel)
                .subscribe_projects(Request::new(SubscribeProjectsRequest {}))
                .await
            {
                Ok(resp) => resp.into_inner(),
                Err(e) => {
                    sender.send(QueryState::Error(e.to_string()));
                    return;
                }
            };

            while let Some(next) = stream.next().await {
                match next
                    .map_err(|e| e.to_string())
                    .and_then(QueryClient::map_projects)
                {
                    Ok(projects) => {
                        sender.send(QueryState::Data(projects));
                    }
                    Err(e) => {
                        sender.send(QueryState::Error(e.to_string()));
                    }
                };
            }

            sender.send(QueryState::Error(
                "projects stream closed unexpectedly".to_string(),
            ));
        });
    }

    pub fn use_projects(&mut self, ui: &Ui) -> QueryState<Projects> {
        if let Some(projects_update) = self.projects_state_inbox.read(ui).last() {
            self.projects_state = projects_update;
        }
        self.projects_state.clone()
    }

    fn map_projects(reply: SubscribeProjectsReply) -> Result<Vec<Project>, String> {
        reply
            .projects
            .into_iter()
            .map(Project::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}
