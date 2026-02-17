use crate::backend::Project;
use tarpc::context;
use uuid::Uuid;

use super::store::{StoreMessage, remove_project, upsert_project};
use super::{Loadable, SyncEngineClient};

impl SyncEngineClient {
    pub fn projects_state(&self) -> Loadable<Vec<Project>> {
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

    pub fn project_state(&self, id: Uuid) -> Loadable<Option<Project>> {
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
                    sender
                        .send(StoreMessage::ProjectError {
                            id: None,
                            message: error.to_string(),
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(StoreMessage::ProjectError {
                            id: None,
                            message: error.to_string(),
                        })
                        .ok();
                }
            }
        });
    }

    pub fn ensure_project_loaded(&self, project_id: Uuid) {
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
                    sender
                        .send(StoreMessage::ProjectError {
                            id: Some(project_id),
                            message: error.to_string(),
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(StoreMessage::ProjectError {
                            id: Some(project_id),
                            message: error.to_string(),
                        })
                        .ok();
                }
            }
        });
    }

    pub fn create_project(&self, project: Project) {
        {
            let mut store = self.store.borrow_mut();
            upsert_project(&mut store, &project);
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
                    sender
                        .send(StoreMessage::ProjectError {
                            id: None,
                            message: error.to_string(),
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(StoreMessage::ProjectError {
                            id: None,
                            message: error.to_string(),
                        })
                        .ok();
                }
            }
        });
    }

    pub fn delete_project(&self, project_id: Uuid) {
        {
            let mut store = self.store.borrow_mut();
            remove_project(&mut store, project_id);
        }

        let client = self.client.clone();
        let sender = self.updates.sender();
        tokio::spawn(async move {
            let result = client.delete_project(context::current(), project_id).await;
            match result {
                Ok(Ok(())) => {
                    sender.send(StoreMessage::ProjectDeleted(project_id)).ok();
                }
                Ok(Err(error)) => {
                    sender
                        .send(StoreMessage::ProjectError {
                            id: Some(project_id),
                            message: error.to_string(),
                        })
                        .ok();
                }
                Err(error) => {
                    sender
                        .send(StoreMessage::ProjectError {
                            id: Some(project_id),
                            message: error.to_string(),
                        })
                        .ok();
                }
            }
        });
    }
}
