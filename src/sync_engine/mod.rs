use crate::backend::{
    BackendServer, Contract, ContractClient, ContractRequest, ContractResponse, Project,
    ProjectsInbox,
};
use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use tarpc::{
    self, ClientMessage, Response, client, context,
    server::{self, BaseChannel, Channel},
    transport::channel::UnboundedChannel,
};

pub struct SyncEngineClient {
    client: ContractClient,

    projects_inbox: ProjectsInbox,
}

impl SyncEngineClient {
    pub fn new() -> Self {
        // Setup inboxes first since we pass them to the backend
        let projects_inbox = UiInbox::new();

        let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

        let server = server::BaseChannel::with_defaults(server_transport);
        tokio::spawn(
            server
                .execute(BackendServer::new(projects_inbox.sender()).serve())
                // Handle all requests concurrently.
                .for_each(|response| async move {
                    tokio::spawn(response);
                }),
        );

        let client = ContractClient::new(client::Config::default(), client_transport).spawn();

        Self {
            client,
            projects_inbox,
        }
    }

    pub fn listen_projects(&self, ui: &Ui) -> impl Iterator<Item = Vec<Project>> {
        return self.projects_inbox.read(ui);
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
