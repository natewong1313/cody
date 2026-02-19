use crate::backend::Project;
use tarpc::context;
use uuid::Uuid;

use super::store::StoreMessage;
use super::{LiveQueryClient, Loadable, QueryKey, flatten_rpc};

impl LiveQueryClient {
    pub fn projects(&self) -> Loadable<Vec<Project>> {
        self.load_projects_if_needed();

        let store = self.store.borrow();
        match &store.projects {
            Loadable::Idle => Loadable::Idle,
            Loadable::Loading => Loadable::Loading,
            Loadable::Error(message) => Loadable::Error(message.clone()),
            Loadable::Ready(ids) => {
                let projects = ids
                    .iter()
                    .filter_map(|id| store.projects_by_id.get(id))
                    .cloned()
                    .collect();
                Loadable::Ready(projects)
            }
        }
    }

    pub fn project(&self, id: Uuid) -> Loadable<Option<Project>> {
        self.load_project_if_needed(id);

        let store = self.store.borrow();
        if let Some(project) = store.projects_by_id.get(&id) {
            return Loadable::Ready(Some(project.clone()));
        }

        store
            .project_states
            .get(&id)
            .cloned()
            .unwrap_or(Loadable::Idle)
    }

    fn load_projects_if_needed(&self) {
        let key = QueryKey::Projects;
        if self.is_in_flight(key) {
            return;
        }

        let should_load = {
            let store = self.store.borrow();
            matches!(store.projects, Loadable::Idle | Loadable::Error(_))
        };

        if !should_load {
            return;
        }

        self.start_query(
            key,
            |store| {
                store.projects = Loadable::Loading;
            },
            |client| async move { client.list_projects(context::current()).await },
            StoreMessage::ProjectsLoaded,
            |message| StoreMessage::ProjectError { id: None, message },
        );
    }

    fn load_project_if_needed(&self, project_id: Uuid) {
        let key = QueryKey::Project(project_id);
        if self.is_in_flight(key) {
            return;
        }

        {
            let store = self.store.borrow();
            if store.projects_by_id.contains_key(&project_id) {
                return;
            }

            if let Some(state) = store.project_states.get(&project_id) {
                if matches!(state, Loadable::Loading | Loadable::Ready(_)) {
                    return;
                }
            }
        }

        self.start_query(
            key,
            move |store| {
                store.project_states.insert(project_id, Loadable::Loading);
            },
            move |client| async move { client.get_project(context::current(), project_id).await },
            move |project| StoreMessage::ProjectLoaded {
                id: project_id,
                project,
            },
            move |message| StoreMessage::ProjectError {
                id: Some(project_id),
                message,
            },
        );
    }

    pub fn create_project(&self, project: Project) {
        let client = self.client.clone();
        let sender = self.updates.sender();

        tokio::spawn(async move {
            let result = flatten_rpc(client.create_project(context::current(), project).await);
            if let Err(message) = result {
                sender
                    .send(StoreMessage::ProjectError { id: None, message })
                    .ok();
            }
        });
    }

    pub fn delete_project(&self, project_id: Uuid) {
        let client = self.client.clone();
        let sender = self.updates.sender();

        tokio::spawn(async move {
            let result = flatten_rpc(client.delete_project(context::current(), project_id).await);
            if let Err(message) = result {
                sender
                    .send(StoreMessage::ProjectError {
                        id: Some(project_id),
                        message,
                    })
                    .ok();
            }
        });
    }
}
