use crate::backend::{
    BackendServer, Project, ProjectInbox, ProjectsInbox,
    mutations::{Mutations, MutationsClient},
};
use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use std::cell::RefCell;
use std::collections::HashMap;
use tarpc::{
    self, client, context,
    server::{self, Channel},
};
use uuid::Uuid;

pub struct SyncEngineClient {
    client: MutationsClient,

    projects_inbox: ProjectsInbox,
    project_inbox: ProjectInbox,
    /// Buffers per-project updates so multiple project pages can coexist
    /// without consuming each other's updates from the shared inbox.
    project_updates: RefCell<HashMap<Uuid, Option<Project>>>,
}

/// This is the middle layer between the backend and the gui
/// its inspired by tanstack query and tanstack db
/// I realized sync engine maybe isn't a great name but whatever
impl SyncEngineClient {
    pub fn new() -> Self {
        // Setup inboxes first since we pass them to the backend
        let projects_inbox = UiInbox::new();
        let project_inbox = UiInbox::new();

        let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

        let server = server::BaseChannel::with_defaults(server_transport);
        tokio::spawn(
            server
                .execute(
                    BackendServer::new(projects_inbox.sender(), project_inbox.sender()).serve(),
                )
                // Handle all requests concurrently.
                .for_each(|response| async move {
                    tokio::spawn(response);
                }),
        );

        let client = MutationsClient::new(client::Config::default(), client_transport).spawn();

        Self {
            client,
            projects_inbox,
            project_inbox,
            project_updates: RefCell::new(HashMap::new()),
        }
    }

    /// listen_* are wrapper functions around inboxes
    pub fn listen_projects(&self, ui: &Ui) -> Option<Vec<Project>> {
        return self.projects_inbox.read(ui).last();
    }
    pub fn listen_project(&self, ui: &Ui, project_id: &Uuid) -> Option<Option<Project>> {
        // Drain all pending updates from the inbox into the per-project buffer.
        // This ensures updates for other project pages aren't lost.
        let mut updates = self.project_updates.borrow_mut();
        for (id, project) in self.project_inbox.read(ui) {
            updates.insert(id, Some(project));
        }
        // Return and remove the update for this specific project, if any.
        updates.remove(project_id)
    }

    pub fn create_project(&self, project: Project) {
        let client = self.client.clone();
        tokio::spawn(async move {
            client
                .create_project(context::current(), project)
                .await
                .unwrap()
        });
    }
}

#[macro_export]
macro_rules! listen {
    ($self:ident, $ui:expr, $listener:expr, $field:ident) => {
        if let Some(updated) = $listener($ui) {
            $self.$field = updated;
        }
    };
}
