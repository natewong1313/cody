use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender, channel},
};

use crate::{
    actions::{ActionContext, handle_action},
    live_query::LiveQueryClient,
    opencode::{OpencodeApiClient, OpencodeSession},
    pages::{PageAction, PageContext, PagesRouter},
};
use egui_inbox::UiInbox;

#[cfg(feature = "local")]
use subsecond;

pub struct App {
    pub api_client: OpencodeApiClient,
    pub live_query: LiveQueryClient,
    pages_router: PagesRouter,

    pub action_sender: Sender<PageAction>,
    action_reciever: Receiver<PageAction>,

    session_inbox: UiInbox<Result<OpencodeSession, String>>,
    pub current_sessions: HashMap<String, OpencodeSession>,
}

impl App {
    pub fn new(api_client: OpencodeApiClient, live_query: LiveQueryClient) -> Self {
        let (action_sender, action_reciever) = channel();
        Self {
            api_client,
            live_query,
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
        while let Ok(action) = self.action_reciever.try_recv() {
            let mut action_ctx = ActionContext {
                pages_router: &mut self.pages_router,
                api_client: &self.api_client,
                session_inbox: &self.session_inbox,
            };
            handle_action(&mut action_ctx, action);
        }

        let mut page_ctx = PageContext {
            api_client: &self.api_client,
            live_query: &self.live_query,
            action_sender: &self.action_sender,
            current_sessions: &self.current_sessions,
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
