use crate::backend::{Project, Session};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Loadable, QueryKey};

#[derive(Debug, Clone)]
pub(super) struct LiveQueryStore {
    pub(super) projects: Loadable<Vec<Uuid>>,
    pub(super) projects_by_id: HashMap<Uuid, Project>,
    pub(super) project_states: HashMap<Uuid, Loadable<Option<Project>>>,
    pub(super) sessions_by_id: HashMap<Uuid, Session>,
    pub(super) session_states: HashMap<Uuid, Loadable<Option<Session>>>,
    pub(super) sessions_by_project_states: HashMap<Uuid, Loadable<Vec<Uuid>>>,
}

impl Default for LiveQueryStore {
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

impl LiveQueryStore {
    pub(super) fn apply(&mut self, message: StoreMessage) -> Vec<QueryKey> {
        match message {
            StoreMessage::ProjectsLoaded(projects) => {
                let ids: Vec<Uuid> = projects.iter().map(|project| project.id).collect();
                for project in &projects {
                    upsert_project(self, project);
                }
                self.projects = Loadable::Ready(ids);
                vec![QueryKey::Projects]
            }
            StoreMessage::ProjectLoaded { id, project } => {
                if let Some(project) = project {
                    self.projects_by_id.insert(id, project.clone());
                    self.project_states
                        .insert(id, Loadable::Ready(Some(project)));
                } else {
                    self.project_states.insert(id, Loadable::Ready(None));
                }
                vec![QueryKey::Project(id)]
            }
            StoreMessage::ProjectUpserted(project) => {
                upsert_project(self, &project);
                vec![]
            }
            StoreMessage::ProjectDeleted(project_id) => {
                remove_project(self, project_id);
                vec![QueryKey::Project(project_id)]
            }
            StoreMessage::ProjectError { id, message } => {
                if let Some(project_id) = id {
                    self.project_states
                        .insert(project_id, Loadable::Error(message));
                    vec![QueryKey::Project(project_id)]
                } else {
                    self.projects = Loadable::Error(message);
                    vec![QueryKey::Projects]
                }
            }
            StoreMessage::SessionsByProjectLoaded {
                project_id,
                sessions,
            } => {
                let ids: Vec<Uuid> = sessions.iter().map(|session| session.id).collect();
                for session in sessions {
                    self.sessions_by_id.insert(session.id, session.clone());
                    self.session_states
                        .insert(session.id, Loadable::Ready(Some(session)));
                }
                self.sessions_by_project_states
                    .insert(project_id, Loadable::Ready(ids));
                vec![QueryKey::SessionsByProject(project_id)]
            }
            StoreMessage::SessionLoaded { id, session } => {
                if let Some(session) = session {
                    upsert_session(self, &session);
                } else {
                    self.session_states.insert(id, Loadable::Ready(None));
                }
                vec![QueryKey::Session(id)]
            }
            StoreMessage::SessionUpserted(session) => {
                upsert_session(self, &session);
                vec![]
            }
            StoreMessage::SessionDeleted(session_id) => {
                remove_session(self, session_id);
                vec![QueryKey::Session(session_id)]
            }
            StoreMessage::SessionError {
                project_id,
                session_id,
                message,
            } => {
                let mut completed = Vec::new();

                if let Some(project_id) = project_id {
                    self.sessions_by_project_states
                        .insert(project_id, Loadable::Error(message.clone()));
                    completed.push(QueryKey::SessionsByProject(project_id));
                }

                if let Some(session_id) = session_id {
                    self.session_states
                        .insert(session_id, Loadable::Error(message));
                    completed.push(QueryKey::Session(session_id));
                }

                completed
            }
        }
    }

    pub(super) fn invalidate(&mut self, key: QueryKey) {
        match key {
            QueryKey::Projects => {
                self.projects = Loadable::Idle;
            }
            QueryKey::Project(id) => {
                self.project_states.insert(id, Loadable::Idle);
            }
            QueryKey::SessionsByProject(project_id) => {
                self.sessions_by_project_states
                    .insert(project_id, Loadable::Idle);
            }
            QueryKey::Session(session_id) => {
                self.session_states.insert(session_id, Loadable::Idle);
            }
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

pub(super) fn upsert_session_into_project_index(store: &mut LiveQueryStore, session: &Session) {
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

pub(super) fn remove_session_from_project_index(store: &mut LiveQueryStore, session: &Session) {
    if let Some(Loadable::Ready(ids)) = store
        .sessions_by_project_states
        .get_mut(&session.project_id)
    {
        ids.retain(|id| *id != session.id);
    }
}

pub(super) fn upsert_project(store: &mut LiveQueryStore, project: &Project) {
    let project_id = project.id;
    store.projects_by_id.insert(project_id, project.clone());
    store
        .project_states
        .insert(project_id, Loadable::Ready(Some(project.clone())));

    if let Loadable::Ready(ids) = &mut store.projects {
        if !ids.iter().any(|existing| *existing == project_id) {
            ids.push(project_id);
        }
    }

    sort_projects_by_updated_at_desc(store);
}

pub(super) fn remove_project(store: &mut LiveQueryStore, project_id: Uuid) {
    store.projects_by_id.remove(&project_id);
    store
        .project_states
        .insert(project_id, Loadable::Ready(None));
    store.sessions_by_project_states.remove(&project_id);

    let session_ids: Vec<Uuid> = store
        .sessions_by_id
        .iter()
        .filter_map(|(id, session)| (session.project_id == project_id).then_some(*id))
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

pub(super) fn upsert_session(store: &mut LiveQueryStore, session: &Session) {
    if let Some(existing) = store.sessions_by_id.get(&session.id).cloned() {
        remove_session_from_project_index(store, &existing);
    }

    store.sessions_by_id.insert(session.id, session.clone());
    store
        .session_states
        .insert(session.id, Loadable::Ready(Some(session.clone())));
    upsert_session_into_project_index(store, session);
    sort_sessions_by_updated_at_desc(store, session.project_id);
}

pub(super) fn remove_session(store: &mut LiveQueryStore, session_id: Uuid) {
    if let Some(existing) = store.sessions_by_id.remove(&session_id) {
        remove_session_from_project_index(store, &existing);
    }
    store
        .session_states
        .insert(session_id, Loadable::Ready(None));
}

fn sort_projects_by_updated_at_desc(store: &mut LiveQueryStore) {
    let updated_at_by_id: HashMap<Uuid, _> = store
        .projects_by_id
        .iter()
        .map(|(id, project)| (*id, project.updated_at))
        .collect();

    if let Loadable::Ready(ids) = &mut store.projects {
        ids.sort_by(|a, b| {
            let a_updated = updated_at_by_id.get(a);
            let b_updated = updated_at_by_id.get(b);
            b_updated.cmp(&a_updated).then_with(|| a.cmp(b))
        });
    }
}

fn sort_sessions_by_updated_at_desc(store: &mut LiveQueryStore, project_id: Uuid) {
    let updated_at_by_id: HashMap<Uuid, _> = store
        .sessions_by_id
        .iter()
        .filter_map(|(id, session)| {
            (session.project_id == project_id).then_some((*id, session.updated_at))
        })
        .collect();

    if let Some(Loadable::Ready(ids)) = store.sessions_by_project_states.get_mut(&project_id) {
        ids.sort_by(|a, b| {
            let a_updated = updated_at_by_id.get(a);
            let b_updated = updated_at_by_id.get(b);
            b_updated.cmp(&a_updated).then_with(|| a.cmp(b))
        });
    }
}
