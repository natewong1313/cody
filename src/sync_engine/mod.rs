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
    StoreMessage, SyncStore, remove_session_from_project_index, upsert_session_into_project_index,
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
            match message {
                StoreMessage::ProjectsLoaded(projects) => {
                    let ids: Vec<uuid::Uuid> = projects.iter().map(|project| project.id).collect();
                    for project in projects {
                        store.projects_by_id.insert(project.id, project.clone());
                        store
                            .project_states
                            .insert(project.id, Loadable::Ready(Some(project)));
                    }
                    store.projects = Loadable::Ready(ids);
                    self.projects_in_flight.set(false);
                }
                StoreMessage::ProjectLoaded { id, project } => {
                    if let Some(project) = project.clone() {
                        store.projects_by_id.insert(id, project);
                    }
                    store.project_states.insert(id, Loadable::Ready(project));
                    self.project_in_flight.borrow_mut().remove(&id);
                }
                StoreMessage::ProjectUpserted(project) => {
                    let id = project.id;
                    store.projects_by_id.insert(id, project.clone());
                    store
                        .project_states
                        .insert(id, Loadable::Ready(Some(project.clone())));

                    if let Loadable::Ready(ids) = &mut store.projects {
                        if !ids.iter().any(|existing| *existing == id) {
                            ids.push(id);
                        }
                    }
                }
                StoreMessage::ProjectDeleted(project_id) => {
                    store.projects_by_id.remove(&project_id);
                    store
                        .project_states
                        .insert(project_id, Loadable::Ready(None));

                    store.sessions_by_project_states.remove(&project_id);
                    let session_ids: Vec<uuid::Uuid> = store
                        .sessions_by_id
                        .iter()
                        .filter_map(|(id, session)| {
                            (session.project_id == project_id).then_some(*id)
                        })
                        .collect();
                    for session_id in session_ids {
                        store.sessions_by_id.remove(&session_id);
                        store
                            .session_states
                            .insert(session_id, Loadable::Ready(None));
                    }

                    if let Loadable::Ready(ids) = &mut store.projects {
                        ids.retain(|id| *id != project_id);
                    }
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
                    if let Some(session) = session.clone() {
                        if let Some(existing) = store.sessions_by_id.get(&id).cloned() {
                            remove_session_from_project_index(&mut store, &existing);
                        }

                        store.sessions_by_id.insert(id, session.clone());
                        upsert_session_into_project_index(&mut store, &session);
                    }

                    store.session_states.insert(id, Loadable::Ready(session));
                    self.session_in_flight.borrow_mut().remove(&id);
                }
                StoreMessage::SessionUpserted(session) => {
                    if let Some(existing) = store.sessions_by_id.get(&session.id).cloned() {
                        remove_session_from_project_index(&mut store, &existing);
                    }

                    store.sessions_by_id.insert(session.id, session.clone());
                    store
                        .session_states
                        .insert(session.id, Loadable::Ready(Some(session.clone())));
                    upsert_session_into_project_index(&mut store, &session);
                }
                StoreMessage::SessionDeleted(session_id) => {
                    if let Some(existing) = store.sessions_by_id.remove(&session_id) {
                        remove_session_from_project_index(&mut store, &existing);
                    }
                    store
                        .session_states
                        .insert(session_id, Loadable::Ready(None));
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
}
