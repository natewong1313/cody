use crate::backend::{
    BackendServer, Project,
    rpc::{BackendRpc, BackendRpcClient},
};
use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use tarpc::{
    self, client, context,
    server::{self, Channel},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Loadable<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

#[derive(Debug, Clone)]
struct ProjectStore {
    projects: Loadable<Vec<Uuid>>,
    by_id: HashMap<Uuid, Project>,
    project_states: HashMap<Uuid, Loadable<Option<Project>>>,
}

impl Default for ProjectStore {
    fn default() -> Self {
        Self {
            projects: Loadable::Idle,
            by_id: HashMap::new(),
            project_states: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum StoreMessage {
    ProjectsLoaded(Vec<Project>),
    ProjectLoaded { id: Uuid, project: Option<Project> },
    ProjectUpserted(Project),
    ProjectDeleted(Uuid),
    Error(String),
}

pub struct SyncEngineClient {
    client: BackendRpcClient,
    store: RefCell<ProjectStore>,
    updates: UiInbox<StoreMessage>,
    projects_in_flight: Cell<bool>,
    project_in_flight: RefCell<HashSet<Uuid>>,
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
            store: RefCell::new(ProjectStore::default()),
            updates,
            projects_in_flight: Cell::new(false),
            project_in_flight: RefCell::new(HashSet::new()),
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
                    let ids: Vec<Uuid> = projects.iter().map(|project| project.id).collect();
                    for project in projects {
                        store.by_id.insert(project.id, project.clone());
                        store
                            .project_states
                            .insert(project.id, Loadable::Ready(Some(project)));
                    }
                    store.projects = Loadable::Ready(ids);
                    self.projects_in_flight.set(false);
                }
                StoreMessage::ProjectLoaded { id, project } => {
                    if let Some(project) = project.clone() {
                        store.by_id.insert(id, project);
                    }
                    store.project_states.insert(id, Loadable::Ready(project));
                    self.project_in_flight.borrow_mut().remove(&id);
                }
                StoreMessage::ProjectUpserted(project) => {
                    let id = project.id;
                    store.by_id.insert(id, project.clone());
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
                    store.by_id.remove(&project_id);
                    store
                        .project_states
                        .insert(project_id, Loadable::Ready(None));

                    if let Loadable::Ready(ids) = &mut store.projects {
                        ids.retain(|id| *id != project_id);
                    }
                }
                StoreMessage::Error(message) => {
                    if self.projects_in_flight.get() {
                        store.projects = Loadable::Error(message.clone());
                        self.projects_in_flight.set(false);
                    }
                }
            }
        }
    }

    pub fn projects_state(&self) -> Loadable<Vec<Project>> {
        let store = self.store.borrow();
        match &store.projects {
            Loadable::Idle => Loadable::Idle,
            Loadable::Loading => Loadable::Loading,
            Loadable::Error(message) => Loadable::Error(message.clone()),
            Loadable::Ready(ids) => {
                let projects = ids
                    .iter()
                    .filter_map(|id| store.by_id.get(id))
                    .cloned()
                    .collect();
                Loadable::Ready(projects)
            }
        }
    }

    pub fn project_state(&self, id: Uuid) -> Loadable<Option<Project>> {
        let store = self.store.borrow();
        if let Some(project) = store.by_id.get(&id) {
            return Loadable::Ready(Some(project.clone()));
        }
        store
            .project_states
            .get(&id)
            .cloned()
            .unwrap_or(Loadable::Idle)
    }

    pub fn ensure_projects_loaded(&self) {
        if self.projects_in_flight.get() {
            return;
        }

        let should_load = {
            let store = self.store.borrow();
            matches!(store.projects, Loadable::Idle | Loadable::Error(_))
        };

        if !should_load {
            return;
        }

        {
            let mut store = self.store.borrow_mut();
            store.projects = Loadable::Loading;
        }
        self.projects_in_flight.set(true);

        let client = self.client.clone();
        let sender = self.updates.sender();
        tokio::spawn(async move {
            let result = client.list_projects(context::current()).await;
            match result {
                Ok(Ok(projects)) => {
                    sender.send(StoreMessage::ProjectsLoaded(projects)).ok();
                }
                Ok(Err(error)) => {
                    sender.send(StoreMessage::Error(error.to_string())).ok();
                }
                Err(error) => {
                    sender.send(StoreMessage::Error(error.to_string())).ok();
                }
            }
        });
    }

    pub fn ensure_project_loaded(&self, project_id: Uuid) {
        {
            let store = self.store.borrow();
            if store.by_id.contains_key(&project_id) {
                return;
            }

            if let Some(state) = store.project_states.get(&project_id) {
                if matches!(state, Loadable::Loading | Loadable::Ready(_)) {
                    return;
                }
            }
        }

        {
            let mut in_flight = self.project_in_flight.borrow_mut();
            if in_flight.contains(&project_id) {
                return;
            }
            in_flight.insert(project_id);
        }

        {
            let mut store = self.store.borrow_mut();
            store.project_states.insert(project_id, Loadable::Loading);
        }

        let client = self.client.clone();
        let sender = self.updates.sender();
        tokio::spawn(async move {
            let result = client.get_project(context::current(), project_id).await;
            match result {
                Ok(Ok(project)) => {
                    sender
                        .send(StoreMessage::ProjectLoaded {
                            id: project_id,
                            project,
                        })
                        .ok();
                }
                Ok(Err(error)) => {
                    sender.send(StoreMessage::Error(error.to_string())).ok();
                }
                Err(error) => {
                    sender.send(StoreMessage::Error(error.to_string())).ok();
                }
            }
        });
    }

    pub fn create_project(&self, project: Project) {
        {
            let mut store = self.store.borrow_mut();
            store.by_id.insert(project.id, project.clone());
            store
                .project_states
                .insert(project.id, Loadable::Ready(Some(project.clone())));
            if let Loadable::Ready(ids) = &mut store.projects {
                if !ids.iter().any(|id| *id == project.id) {
                    ids.push(project.id);
                }
            }
        }

        let client = self.client.clone();
        let sender = self.updates.sender();
        tokio::spawn(async move {
            let result = client.create_project(context::current(), project).await;
            match result {
                Ok(Ok(created)) => {
                    sender.send(StoreMessage::ProjectUpserted(created)).ok();
                }
                Ok(Err(error)) => {
                    sender.send(StoreMessage::Error(error.to_string())).ok();
                }
                Err(error) => {
                    sender.send(StoreMessage::Error(error.to_string())).ok();
                }
            }
        });
    }

    pub fn delete_project(&self, project_id: Uuid) {
        {
            let mut store = self.store.borrow_mut();
            store.by_id.remove(&project_id);
            store
                .project_states
                .insert(project_id, Loadable::Ready(None));
            if let Loadable::Ready(ids) = &mut store.projects {
                ids.retain(|id| *id != project_id);
            }
        }

        let client = self.client.clone();
        let sender = self.updates.sender();
        tokio::spawn(async move {
            let result = client.delete_project(context::current(), project_id).await;
            if let Err(error) = result {
                sender.send(StoreMessage::Error(error.to_string())).ok();
                return;
            }
            sender.send(StoreMessage::ProjectDeleted(project_id)).ok();
        });
    }
}
