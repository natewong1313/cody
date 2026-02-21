use std::cmp::Ordering;

use rusqlite::Connection;
use tokio::sync::watch;
use uuid::Uuid;

use crate::backend::{
    BackendContext, Harness, Project, Session,
    data::{
        project::{ProjectRepo, ProjectRepoError},
        session::{SessionRepo, SessionRepoError},
    },
    harness::opencode::OpencodeHarness,
    state::{EntityState, GroupedState},
};

pub struct LocalBackend {
    project_repo: ProjectRepo,
    session_repo: SessionRepo,
    project_state: EntityState<Uuid, Project>,
    session_state: EntityState<Uuid, Session>,
    session_by_project_state: GroupedState<Uuid, Uuid, Session>,
}

#[derive(thiserror::Error, Debug)]
pub enum LocalBackendStartupError {
    #[error("Error establishing connection {0}")]
    Connection(#[from] rusqlite::Error),
    #[error("Error migrating db {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("Harness initialization failed: {0}")]
    Harness(String),
    #[error("Project repo initialization failed: {0}")]
    ProjectRepo(#[from] ProjectRepoError),
    #[error("Session repo initialization failed: {0}")]
    SessionRepo(#[from] SessionRepoError),
}

impl LocalBackend {
    fn new() -> Result<Self, LocalBackendStartupError> {
        let conn = Connection::open("./cody.db")?;
        let harness =
            OpencodeHarness::new().map_err(|e| LocalBackendStartupError::Harness(e.to_string()))?;
        let ctx = BackendContext::new(conn, harness);

        let project_repo = ProjectRepo::new(ctx.clone());
        let initial_projects = project_repo.list()?;
        let project_state = EntityState::new(
            "projects",
            initial_projects,
            |project| project.id,
            sort_projects,
        );
        let session_repo = SessionRepo::new(ctx.clone());
        let initial_sessions = session_repo.list()?;
        let session_state = EntityState::new(
            "sessions",
            initial_sessions.clone(),
            |session| session.id,
            sort_sessions,
        );
        let session_by_project_state = GroupedState::new(
            "sessions_by_project",
            initial_sessions,
            |session| session.project_id,
            |session| session.id,
            sort_sessions,
        );

        Ok(Self {
            project_repo,
            session_repo,
            project_state,
            session_state,
            session_by_project_state,
        })
    }
}

fn sort_projects(a: &Project, b: &Project) -> Ordering {
    b.updated_at.cmp(&a.updated_at)
}

fn sort_sessions(a: &Session, b: &Session) -> Ordering {
    b.updated_at.cmp(&a.updated_at)
}

/// This is lowk tedium
impl super::Backend for LocalBackend {
    async fn subscribe_projects(
        &self,
    ) -> Result<watch::Receiver<Vec<super::Project>>, super::BackendError> {
        Ok(self.project_state.subscribe_all())
    }

    async fn subscribe_project(
        &self,
        project_id: Uuid,
    ) -> Result<watch::Receiver<Option<Project>>, super::BackendError> {
        self.project_state
            .subscribe_one(project_id)
            .map_err(Into::into)
    }

    async fn subscribe_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<watch::Receiver<Vec<Session>>, super::BackendError> {
        self.session_by_project_state
            .subscribe_group(project_id)
            .map_err(Into::into)
    }

    async fn subscribe_session(
        &self,
        session_id: Uuid,
    ) -> Result<watch::Receiver<Option<Session>>, super::BackendError> {
        self.session_state
            .subscribe_one(session_id)
            .map_err(Into::into)
    }

    async fn list_projects(
        &self,
    ) -> Result<Vec<super::data::project::Project>, super::BackendError> {
        self.project_state.list().map_err(Into::into)
    }

    async fn get_project(
        &self,
        project_id: Uuid,
    ) -> Result<Option<super::data::project::Project>, super::BackendError> {
        self.project_state.get(project_id).map_err(Into::into)
    }

    async fn create_project(
        &self,
        project: super::data::project::Project,
    ) -> Result<super::data::project::Project, super::BackendError> {
        let project = self.project_repo.create(&project)?;
        self.project_state.upsert(project.clone())?;
        Ok(project)
    }

    async fn update_project(
        &self,
        project: super::data::project::Project,
    ) -> Result<super::data::project::Project, super::BackendError> {
        let project = self.project_repo.update(&project)?;
        self.project_state.upsert(project.clone())?;
        Ok(project)
    }

    async fn delete_project(&self, project_id: Uuid) -> Result<(), super::BackendError> {
        self.project_repo.delete(&project_id)?;
        self.project_state.remove(project_id)?;

        let sessions = self.session_by_project_state.list_group(project_id)?;
        for session in sessions {
            self.session_state.remove(session.id)?;
        }
        self.session_by_project_state.remove_group(project_id)?;

        Ok(())
    }

    async fn list_sessions_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<super::data::session::Session>, super::BackendError> {
        self.session_by_project_state
            .list_group(project_id)
            .map_err(Into::into)
    }

    async fn get_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<super::data::session::Session>, super::BackendError> {
        self.session_state.get(session_id).map_err(Into::into)
    }

    async fn create_session(
        &self,
        session: super::data::session::Session,
    ) -> Result<super::data::session::Session, super::BackendError> {
        let session = self.session_repo.create(&session).await?;
        self.session_state.upsert(session.clone())?;
        self.session_by_project_state.upsert(session.clone())?;
        Ok(session)
    }

    async fn update_session(
        &self,
        session: super::data::session::Session,
    ) -> Result<super::data::session::Session, super::BackendError> {
        let session = self.session_repo.update(&session)?;
        self.session_state.upsert(session.clone())?;
        self.session_by_project_state.upsert(session.clone())?;
        Ok(session)
    }

    async fn delete_session(&self, session_id: Uuid) -> Result<(), super::BackendError> {
        self.session_repo.delete(&session_id)?;
        self.session_state.remove(session_id)?;
        self.session_by_project_state.remove(session_id)?;
        Ok(())
    }
}
