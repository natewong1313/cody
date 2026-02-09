use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{
    actions::{handle_action, ActionContext},
    opencode::{OpencodeApiClient, OpencodeSession},
    pages::{PageAction, PageContext, PageType, PagesRouter},
};
use egui_inbox::UiInbox;

pub struct App {
    pub api_client: OpencodeApiClient,
    pages_router: PagesRouter,

    pub action_sender: Sender<PageAction>,
    action_reciever: Receiver<PageAction>,

    session_inbox: UiInbox<Result<OpencodeSession, String>>,
    pub current_sessions: HashMap<String, OpencodeSession>,
}

impl App {
    pub fn new(api_client: OpencodeApiClient) -> Self {
        let (action_sender, action_reciever) = channel();
        Self {
            api_client,
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
            action_sender: &self.action_sender,
            current_sessions: &self.current_sessions,
        };
        self.pages_router.mount(ctx, &mut page_ctx);
    }
}
