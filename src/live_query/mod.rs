mod projects;
mod sessions;
mod store;

use crate::backend::rpc::{BackendRpc, BackendRpcClient, RpcError};
use crate::backend::{BackendEvent, BackendServer};
use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use std::cell::RefCell;
use std::collections::HashSet;
use std::future::Future;
use store::{LiveQueryStore, StoreMessage};
use tarpc::{
    self, client,
    server::{self, Channel},
};
use tokio::sync::broadcast;
use uuid::Uuid;

fn flatten_rpc<T>(
    result: Result<Result<T, RpcError>, tarpc::client::RpcError>,
) -> Result<T, String> {
    result
        .map_err(|e| e.to_string())
        .and_then(|r| r.map_err(|e| e.to_string()))
}

#[derive(Debug, Clone)]
pub enum Loadable<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum QueryKey {
    Projects,
    Project(Uuid),
    SessionsByProject(Uuid),
    Session(Uuid),
}

pub struct LiveQueryClient {
    client: BackendRpcClient,
    store: RefCell<LiveQueryStore>,
    updates: UiInbox<StoreMessage>,
    in_flight: RefCell<HashSet<QueryKey>>,
}

impl LiveQueryClient {
    pub fn new() -> Self {
        let updates = UiInbox::new();
        let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

        let (sender, _) = broadcast::channel::<BackendEvent>(128);
        let server_impl = BackendServer::new(sender);
        let mut event_receiver = server_impl.subscribe();

        let server = server::BaseChannel::with_defaults(server_transport);
        tokio::spawn(
            server
                .execute(server_impl.serve())
                .for_each(|response| async move {
                    tokio::spawn(response);
                }),
        );

        let updates_sender = updates.sender();
        tokio::spawn(async move {
            loop {
                match event_receiver.recv().await {
                    Ok(BackendEvent::ProjectUpserted(project)) => {
                        updates_sender
                            .send(StoreMessage::ProjectUpserted(project))
                            .ok();
                    }
                    Ok(BackendEvent::ProjectDeleted(project_id)) => {
                        updates_sender
                            .send(StoreMessage::ProjectDeleted(project_id))
                            .ok();
                    }
                    Ok(BackendEvent::SessionUpserted(session)) => {
                        updates_sender
                            .send(StoreMessage::SessionUpserted(session))
                            .ok();
                    }
                    Ok(BackendEvent::SessionDeleted(session_id)) => {
                        updates_sender
                            .send(StoreMessage::SessionDeleted(session_id))
                            .ok();
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        let client = BackendRpcClient::new(client::Config::default(), client_transport).spawn();

        Self {
            client,
            store: RefCell::new(LiveQueryStore::default()),
            updates,
            in_flight: RefCell::new(HashSet::new()),
        }
    }

    pub fn poll(&self, ui: &Ui) {
        let messages: Vec<StoreMessage> = self.updates.read(ui).collect();
        if messages.is_empty() {
            return;
        }

        let mut store = self.store.borrow_mut();
        for message in messages {
            let completed = store.apply(message);
            for key in completed {
                self.complete_query(key);
            }
        }
    }

    pub fn invalidate_projects(&self) {
        self.complete_query(QueryKey::Projects);
        self.store.borrow_mut().invalidate(QueryKey::Projects);
    }

    pub fn invalidate_project(&self, id: Uuid) {
        self.complete_query(QueryKey::Project(id));
        self.store.borrow_mut().invalidate(QueryKey::Project(id));
    }

    pub fn invalidate_sessions_by_project(&self, project_id: Uuid) {
        self.complete_query(QueryKey::SessionsByProject(project_id));
        self.store
            .borrow_mut()
            .invalidate(QueryKey::SessionsByProject(project_id));
    }

    pub fn invalidate_session(&self, session_id: Uuid) {
        self.complete_query(QueryKey::Session(session_id));
        self.store
            .borrow_mut()
            .invalidate(QueryKey::Session(session_id));
    }

    fn complete_query(&self, key: QueryKey) {
        self.in_flight.borrow_mut().remove(&key);
    }

    fn is_in_flight(&self, key: QueryKey) -> bool {
        self.in_flight.borrow().contains(&key)
    }

    fn start_query<T, Fut, RpcCall, OkMap, ErrMap>(
        &self,
        key: QueryKey,
        set_loading: impl FnOnce(&mut LiveQueryStore),
        rpc_call: RpcCall,
        ok_map: OkMap,
        err_map: ErrMap,
    ) where
        T: Send + 'static,
        Fut: Future<Output = Result<Result<T, RpcError>, tarpc::client::RpcError>> + Send + 'static,
        RpcCall: FnOnce(BackendRpcClient) -> Fut + Send + 'static,
        OkMap: FnOnce(T) -> StoreMessage + Send + 'static,
        ErrMap: FnOnce(String) -> StoreMessage + Send + 'static,
    {
        {
            let mut in_flight = self.in_flight.borrow_mut();
            if in_flight.contains(&key) {
                return;
            }
            in_flight.insert(key);
        }

        {
            let mut store = self.store.borrow_mut();
            set_loading(&mut store);
        }

        let client = self.client.clone();
        let sender = self.updates.sender();
        tokio::spawn(async move {
            let result = flatten_rpc(rpc_call(client).await);
            match result {
                Ok(payload) => {
                    sender.send(ok_map(payload)).ok();
                }
                Err(message) => {
                    sender.send(err_map(message)).ok();
                }
            }
        });
    }
}
