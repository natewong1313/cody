use crate::backend::Session;
use tarpc::context;
use uuid::Uuid;

use super::store::StoreMessage;
use super::{LiveQueryClient, Loadable, QueryKey, flatten_rpc};

impl LiveQueryClient {
    pub fn sessions_by_project(&self, project_id: Uuid) -> Loadable<Vec<Session>> {
        self.load_sessions_by_project_if_needed(project_id);

        let store = self.store.borrow();
        match store.sessions_by_project_states.get(&project_id) {
            None => Loadable::Idle,
            Some(Loadable::Idle) => Loadable::Idle,
            Some(Loadable::Loading) => Loadable::Loading,
            Some(Loadable::Error(message)) => Loadable::Error(message.clone()),
            Some(Loadable::Ready(ids)) => {
                let sessions = ids
                    .iter()
                    .filter_map(|id| store.sessions_by_id.get(id))
                    .cloned()
                    .collect();
                Loadable::Ready(sessions)
            }
        }
    }

    pub fn session(&self, session_id: Uuid) -> Loadable<Option<Session>> {
        self.load_session_if_needed(session_id);

        let store = self.store.borrow();
        if let Some(session) = store.sessions_by_id.get(&session_id) {
            return Loadable::Ready(Some(session.clone()));
        }

        store
            .session_states
            .get(&session_id)
            .cloned()
            .unwrap_or(Loadable::Idle)
    }

    fn load_sessions_by_project_if_needed(&self, project_id: Uuid) {
        let key = QueryKey::SessionsByProject(project_id);
        if self.is_in_flight(key) {
            return;
        }

        {
            let store = self.store.borrow();
            if let Some(state) = store.sessions_by_project_states.get(&project_id) {
                if matches!(state, Loadable::Loading | Loadable::Ready(_)) {
                    return;
                }
            }
        }

        self.start_query(
            key,
            move |store| {
                store
                    .sessions_by_project_states
                    .insert(project_id, Loadable::Loading);
            },
            move |client| async move {
                client
                    .list_sessions_by_project(context::current(), project_id)
                    .await
            },
            move |sessions| StoreMessage::SessionsByProjectLoaded {
                project_id,
                sessions,
            },
            move |message| StoreMessage::SessionError {
                project_id: Some(project_id),
                session_id: None,
                message,
            },
        );
    }

    fn load_session_if_needed(&self, session_id: Uuid) {
        let key = QueryKey::Session(session_id);
        if self.is_in_flight(key) {
            return;
        }

        {
            let store = self.store.borrow();
            if store.sessions_by_id.contains_key(&session_id) {
                return;
            }

            if let Some(state) = store.session_states.get(&session_id) {
                if matches!(state, Loadable::Loading | Loadable::Ready(_)) {
                    return;
                }
            }
        }

        self.start_query(
            key,
            move |store| {
                store.session_states.insert(session_id, Loadable::Loading);
            },
            move |client| async move { client.get_session(context::current(), session_id).await },
            move |session| StoreMessage::SessionLoaded {
                id: session_id,
                session,
            },
            move |message| StoreMessage::SessionError {
                project_id: None,
                session_id: Some(session_id),
                message,
            },
        );
    }

    pub fn create_session(&self, session: Session) {
        let client = self.client.clone();
        let sender = self.updates.sender();

        tokio::spawn(async move {
            let result = flatten_rpc(client.create_session(context::current(), session).await);
            if let Err(message) = result {
                sender
                    .send(StoreMessage::SessionError {
                        project_id: None,
                        session_id: None,
                        message,
                    })
                    .ok();
            }
        });
    }

    pub fn update_session(&self, session: Session) {
        let session_id = session.id;
        let client = self.client.clone();
        let sender = self.updates.sender();

        tokio::spawn(async move {
            let result = flatten_rpc(client.update_session(context::current(), session).await);
            if let Err(message) = result {
                sender
                    .send(StoreMessage::SessionError {
                        project_id: None,
                        session_id: Some(session_id),
                        message,
                    })
                    .ok();
            }
        });
    }

    pub fn delete_session(&self, session_id: Uuid) {
        let client = self.client.clone();
        let sender = self.updates.sender();

        tokio::spawn(async move {
            let result = flatten_rpc(client.delete_session(context::current(), session_id).await);
            if let Err(message) = result {
                sender
                    .send(StoreMessage::SessionError {
                        project_id: None,
                        session_id: Some(session_id),
                        message,
                    })
                    .ok();
            }
        });
    }
}
