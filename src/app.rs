use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{
    actions::{handle_action, ActionContext},
    opencode::{OpencodeApiClient, OpencodeSession},
    pages::{PageAction, PageContext, PageType, PagesRouter},
    sync_engine::SyncEngineClient,
};
use egui_inbox::UiInbox;

#[cfg(feature = "local")]
use subsecond;

pub struct App {
    pub api_client: OpencodeApiClient,
    pub sync_engine: SyncEngineClient,
    pages_router: PagesRouter,

    pub action_sender: Sender<PageAction>,
    action_reciever: Receiver<PageAction>,

    session_inbox: UiInbox<Result<OpencodeSession, String>>,
    pub current_sessions: HashMap<String, OpencodeSession>,
}

impl App {
    pub fn new(api_client: OpencodeApiClient, sync_engine: SyncEngineClient) -> Self {
        let (action_sender, action_reciever) = channel();
        Self {
            api_client,
            sync_engine,
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
        // Process async results from inbox
        for result in self.session_inbox.read(ctx) {
            match result {
                Ok(session) => {
                    log::info!("created session: {:?}", session);
                    let session_id = session.id.clone();
                    self.current_sessions.insert(session_id.clone(), session);
                    self.pages_router.navigate(PageType::Session(session_id));
                }
                Err(e) => {
                    log::error!("Failed to create session: {}", e);
                }
            }
        }

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
            sync_engine: &self.sync_engine,
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
