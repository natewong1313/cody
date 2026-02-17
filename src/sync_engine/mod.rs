mod projects;
mod sessions;
mod store;

use crate::backend::BackendServer;
use crate::backend::rpc::{BackendRpc, BackendRpcClient};
use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use tarpc::{
    self, client,
    server::{self, Channel},
};

use store::{
    StoreMessage, SyncStore, remove_project, remove_session, upsert_project, upsert_session,
};

#[derive(Debug, Clone)]
pub enum Loadable<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

pub struct SyncEngineClient {
    client: BackendRpcClient,
    store: RefCell<SyncStore>,
    updates: UiInbox<StoreMessage>,
    projects_in_flight: Cell<bool>,
    project_in_flight: RefCell<HashSet<uuid::Uuid>>,
    sessions_by_project_in_flight: RefCell<HashSet<uuid::Uuid>>,
    session_in_flight: RefCell<HashSet<uuid::Uuid>>,
}

impl SyncEngineClient {
    pub fn new() -> Self {
        let updates = UiInbox::new();
        let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

        let server = server::BaseChannel::with_defaults(server_transport);
        tokio::spawn(server.execute(BackendServer::new().serve()).for_each(
            |response| async move {
                tokio::spawn(response);
            },
        ));

        let client = BackendRpcClient::new(client::Config::default(), client_transport).spawn();

        Self {
            client,
            store: RefCell::new(SyncStore::default()),
            updates,
            projects_in_flight: Cell::new(false),
            project_in_flight: RefCell::new(HashSet::new()),
            sessions_by_project_in_flight: RefCell::new(HashSet::new()),
            session_in_flight: RefCell::new(HashSet::new()),
        }
    }

    pub fn poll(&self, ui: &Ui) {
        let messages: Vec<StoreMessage> = self.updates.read(ui).collect();
        if messages.is_empty() {
            return;
        }

        let mut store = self.store.borrow_mut();
        for message in messages {
            self.apply_store_message(&mut store, message);
        }
    }

    fn apply_store_message(&self, store: &mut SyncStore, message: StoreMessage) {
        match message {
            StoreMessage::ProjectsLoaded(projects) => {
                let ids: Vec<uuid::Uuid> = projects.iter().map(|project| project.id).collect();
                for project in &projects {
                    upsert_project(store, project);
                }
                store.projects = Loadable::Ready(ids);
                self.projects_in_flight.set(false);
            }
            StoreMessage::ProjectLoaded { id, project } => {
                if let Some(project) = project {
                    store.projects_by_id.insert(id, project.clone());
                    store
                        .project_states
                        .insert(id, Loadable::Ready(Some(project)));
                } else {
                    store.project_states.insert(id, Loadable::Ready(None));
                }
                self.project_in_flight.borrow_mut().remove(&id);
            }
            StoreMessage::ProjectUpserted(project) => {
                upsert_project(store, &project);
            }
            StoreMessage::ProjectDeleted(project_id) => {
                remove_project(store, project_id);
            }
            StoreMessage::ProjectError { id, message } => {
                if let Some(project_id) = id {
                    store
                        .project_states
                        .insert(project_id, Loadable::Error(message.clone()));
                    self.project_in_flight.borrow_mut().remove(&project_id);
                } else if self.projects_in_flight.get() {
                    store.projects = Loadable::Error(message.clone());
                    self.projects_in_flight.set(false);
                }
            }
            StoreMessage::SessionsByProjectLoaded {
                project_id,
                sessions,
            } => {
                let ids: Vec<uuid::Uuid> = sessions.iter().map(|session| session.id).collect();
                for session in sessions {
                    store.sessions_by_id.insert(session.id, session.clone());
                    store
                        .session_states
                        .insert(session.id, Loadable::Ready(Some(session)));
                }
                store
                    .sessions_by_project_states
                    .insert(project_id, Loadable::Ready(ids));
                self.sessions_by_project_in_flight
                    .borrow_mut()
                    .remove(&project_id);
            }
            StoreMessage::SessionLoaded { id, session } => {
                if let Some(session) = session {
                    upsert_session(store, &session);
                } else {
                    store.session_states.insert(id, Loadable::Ready(None));
                }
                self.session_in_flight.borrow_mut().remove(&id);
            }
            StoreMessage::SessionUpserted(session) => {
                upsert_session(store, &session);
            }
            StoreMessage::SessionDeleted(session_id) => {
                remove_session(store, session_id);
                self.session_in_flight.borrow_mut().remove(&session_id);
            }
            StoreMessage::SessionError {
                project_id,
                session_id,
                message,
            } => {
                if let Some(project_id) = project_id {
                    store
                        .sessions_by_project_states
                        .insert(project_id, Loadable::Error(message.clone()));
                    self.sessions_by_project_in_flight
                        .borrow_mut()
                        .remove(&project_id);
                }

                if let Some(session_id) = session_id {
                    store
                        .session_states
                        .insert(session_id, Loadable::Error(message));
                    self.session_in_flight.borrow_mut().remove(&session_id);
                }
            }
        }
    }
}
