use std::collections::{HashMap, HashSet};

use egui::Ui;
use egui_inbox::UiInbox;
use tonic::{Request, transport::Channel};
use uuid::Uuid;

use crate::backend::{
    ListSessionsByProjectReply, ListSessionsByProjectRequest, Session, SessionClient,
};

use super::QueryState;

pub type SessionsState = QueryState<Vec<Session>>;

pub struct Sessions {
    backend_channel: Channel,
    state_by_project: HashMap<Uuid, SessionsState>,
    is_fetching: HashSet<Uuid>,
    inbox: UiInbox<(Uuid, SessionsState)>,
}

impl Sessions {
    pub fn new(backend_channel: Channel) -> Self {
        Self {
            backend_channel,
            state_by_project: HashMap::new(),
            is_fetching: HashSet::new(),
            inbox: UiInbox::new(),
        }
    }

    pub fn subscribe_state(&mut self, ui: &Ui, project_id: Uuid) -> SessionsState {
        for (updated_project_id, updated_state) in self.inbox.read(ui) {
            self.is_fetching.remove(&updated_project_id);
            self.state_by_project
                .insert(updated_project_id, updated_state);
        }

        self.fetch_if_needed(project_id);

        self.state_by_project
            .get(&project_id)
            .cloned()
            .unwrap_or(QueryState::Loading)
    }

    fn fetch_if_needed(&mut self, project_id: Uuid) {
        if self.is_fetching.contains(&project_id) {
            return;
        }

        if matches!(
            self.state_by_project.get(&project_id),
            Some(QueryState::Data(_))
        ) {
            return;
        }

        self.is_fetching.insert(project_id);
        self.state_by_project
            .insert(project_id, QueryState::Loading);

        let sender = self.inbox.sender().clone();
        let channel = self.backend_channel.clone();

        tokio::spawn(async move {
            let response = SessionClient::new(channel)
                .list_sessions_by_project(Request::new(ListSessionsByProjectRequest {
                    project_id: project_id.to_string(),
                }))
                .await;

            let state = match response {
                Ok(resp) => match Sessions::map(resp.into_inner()) {
                    Ok(sessions) => QueryState::Data(sessions),
                    Err(e) => QueryState::Error(e),
                },
                Err(e) => QueryState::Error(e.to_string()),
            };

            let _ = sender.send((project_id, state));
        });
    }

    fn map(reply: ListSessionsByProjectReply) -> Result<Vec<Session>, String> {
        reply
            .sessions
            .into_iter()
            .map(Session::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}
