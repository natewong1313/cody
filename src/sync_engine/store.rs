use crate::backend::{Project, Session};
use std::collections::HashMap;
use uuid::Uuid;

use super::Loadable;

#[derive(Debug, Clone)]
pub(super) struct SyncStore {
    pub(super) projects: Loadable<Vec<Uuid>>,
    pub(super) projects_by_id: HashMap<Uuid, Project>,
    pub(super) project_states: HashMap<Uuid, Loadable<Option<Project>>>,
    pub(super) sessions_by_id: HashMap<Uuid, Session>,
    pub(super) session_states: HashMap<Uuid, Loadable<Option<Session>>>,
    pub(super) sessions_by_project_states: HashMap<Uuid, Loadable<Vec<Uuid>>>,
}

impl Default for SyncStore {
    fn default() -> Self {
        Self {
            projects: Loadable::Idle,
            projects_by_id: HashMap::new(),
            project_states: HashMap::new(),
            sessions_by_id: HashMap::new(),
            session_states: HashMap::new(),
            sessions_by_project_states: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum StoreMessage {
    ProjectsLoaded(Vec<Project>),
    ProjectLoaded {
        id: Uuid,
        project: Option<Project>,
    },
    ProjectUpserted(Project),
    ProjectDeleted(Uuid),
    ProjectError {
        id: Option<Uuid>,
        message: String,
    },
    SessionsByProjectLoaded {
        project_id: Uuid,
        sessions: Vec<Session>,
    },
    SessionLoaded {
        id: Uuid,
        session: Option<Session>,
    },
    SessionUpserted(Session),
    SessionDeleted(Uuid),
    SessionError {
        project_id: Option<Uuid>,
        session_id: Option<Uuid>,
        message: String,
    },
}

pub(super) fn upsert_session_into_project_index(store: &mut SyncStore, session: &Session) {
    match store
        .sessions_by_project_states
        .get_mut(&session.project_id)
    {
        Some(Loadable::Ready(ids)) => {
            if !ids.iter().any(|id| *id == session.id) {
                ids.push(session.id);
            }
        }
        Some(state) if matches!(state, Loadable::Idle | Loadable::Error(_)) => {
            *state = Loadable::Ready(vec![session.id]);
        }
        None => {
            store
                .sessions_by_project_states
                .insert(session.project_id, Loadable::Ready(vec![session.id]));
        }
        _ => {}
    }
}

pub(super) fn remove_session_from_project_index(store: &mut SyncStore, session: &Session) {
    if let Some(Loadable::Ready(ids)) = store
        .sessions_by_project_states
        .get_mut(&session.project_id)
    {
        ids.retain(|id| *id != session.id);
    }
}
