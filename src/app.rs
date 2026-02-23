use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender, channel},
};

use crate::{
    actions::{ActionContext, handle_action},
    opencode::{OpencodeApiClient, OpencodeSession},
    pages::{PageAction, PageContext, PagesRouter},
};
use egui_inbox::UiInbox;

#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
use crate::{backend::rpc::BackendRpcClient, query::QueryClient};

#[cfg(feature = "local")]
use subsecond;

pub struct App {
    pages_router: PagesRouter,

    pub action_sender: Sender<PageAction>,
    action_reciever: Receiver<PageAction>,

    session_inbox: UiInbox<Result<OpencodeSession, String>>,
    pub current_sessions: HashMap<String, OpencodeSession>,

    #[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
    query_client: QueryClient,
}

impl App {
    #[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
    pub fn new(backend_client: BackendRpcClient) -> Self {
        let (action_sender, action_reciever) = channel();
        Self {
            pages_router: PagesRouter::new(),

            action_sender,
            action_reciever,

            session_inbox: UiInbox::new(),
            current_sessions: HashMap::new(),
            query_client: QueryClient::new(backend_client),
        }
    }

    #[cfg(all(feature = "browser", target_arch = "wasm32"))]
    pub fn new() -> Self {
        let (action_sender, action_reciever) = channel();
        Self {
            pages_router: PagesRouter::new(),

            action_sender,
            action_reciever,

            session_inbox: UiInbox::new(),
            current_sessions: HashMap::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // self.query_client.load_projects_if_needed();
        // self.query_client.poll(ctx);

        while let Ok(action) = self.action_reciever.try_recv() {
            let mut action_ctx = ActionContext {
                pages_router: &mut self.pages_router,
                session_inbox: &self.session_inbox,
            };
            handle_action(&mut action_ctx, action);
        }

        let mut page_ctx = PageContext {
            action_sender: &self.action_sender,
            current_sessions: &self.current_sessions,
            #[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
            query: &mut self.query_client,
        };

        // Wrap rendering in subsecond::call() for hot-reloading support
        // This allows changes in pages/ to be hot-patched without restart
        #[cfg(feature = "local")]
        subsecond::call(|| {
            self.pages_router.mount(ctx, &mut page_ctx);
        });

        #[cfg(not(feature = "local"))]
        self.pages_router.mount(ctx, &mut page_ctx);
    }
}
