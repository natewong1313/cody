use crate::backend::Session;
use tarpc::context;
use uuid::Uuid;

use super::store::{remove_session, upsert_session, StoreMessage};
use super::{flatten_rpc, Loadable, SyncEngineClient};

impl SyncEngineClient {
    pub fn sessions_by_project_state(&self, project_id: Uuid) -> Loadable<Vec<Session>> {
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

    pub fn session_state(&self, session_id: Uuid) -> Loadable<Option<Session>> {
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

    pub fn ensure_sessions_by_project_loaded(&self, project_id: Uuid) {
        {
            let store = self.store.borrow();
            if let Some(state) = store.sessions_by_project_states.get(&project_id) {
                if matches!(state, Loadable::Loading | Loadable::Ready(_)) {
                    return;
                }
            }
        }

        {
            let mut in_flight = self.sessions_by_project_in_flight.borrow_mut();
            if in_flight.contains(&project_id) {
                return;
            }
            in_flight.insert(project_id);
        }

        {
            let mut store = self.store.borrow_mut();
            store
                .sessions_by_project_states
                .insert(project_id, Loadable::Loading);
        }

        spawn_rpc!(self,
            |client| client.list_sessions_by_project(context::current(), project_id).await,
            ok(sessions) => StoreMessage::SessionsByProjectLoaded { project_id, sessions },
            err(message) => StoreMessage::SessionError {
                project_id: Some(project_id),
                session_id: None,
                message,
            },
        );
    }

    pub fn ensure_session_loaded(&self, session_id: Uuid) {
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

        {
            let mut in_flight = self.session_in_flight.borrow_mut();
            if in_flight.contains(&session_id) {
                return;
            }
            in_flight.insert(session_id);
        }

        {
            let mut store = self.store.borrow_mut();
            store.session_states.insert(session_id, Loadable::Loading);
        }

        spawn_rpc!(self,
            |client| client.get_session(context::current(), session_id).await,
            ok(session) => StoreMessage::SessionLoaded { id: session_id, session },
            err(message) => StoreMessage::SessionError {
                project_id: None,
                session_id: Some(session_id),
                message,
            },
        );
    }

    pub fn create_session(&self, session: Session) {
        {
            let mut store = self.store.borrow_mut();
            upsert_session(&mut store, &session);
        }

        spawn_rpc!(self,
            |client| client.create_session(context::current(), session).await,
            ok(created) => StoreMessage::SessionUpserted(created),
            err(message) => StoreMessage::SessionError {
                project_id: None,
                session_id: None,
                message,
            },
        );
    }

    pub fn update_session(&self, session: Session) {
        {
            let mut store = self.store.borrow_mut();
            upsert_session(&mut store, &session);
        }

        spawn_rpc!(self,
            |client| client.update_session(context::current(), session).await,
            ok(updated) => StoreMessage::SessionUpserted(updated),
            err(message) => StoreMessage::SessionError {
                project_id: None,
                session_id: None,
                message,
            },
        );
    }

    pub fn delete_session(&self, session_id: Uuid) {
        {
            let mut store = self.store.borrow_mut();
            remove_session(&mut store, session_id);
        }

        spawn_rpc!(self,
            |client| client.delete_session(context::current(), session_id).await,
            ok(()) => StoreMessage::SessionDeleted(session_id),
            err(message) => StoreMessage::SessionError {
                project_id: None,
                session_id: Some(session_id),
                message,
            },
        );
    }
}
