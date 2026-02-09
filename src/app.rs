use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender, channel},
};

use crate::{
    opencode::{OpencodeApiClient, OpencodeSession, PartInput, SendMessageRequest},
    pages::{PageAction, PageContext, PageType, PagesRouter},
};
use egui::{Button, TextEdit};
use egui_flex::{Flex, item};
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

    fn handle_action(&mut self, action: PageAction) {
        match action {
            PageAction::Navigate(page) => self.pages_router.navigate(page),
            PageAction::CreateSession => {
                let sender = self.session_inbox.sender();
                let client = self.api_client.clone();
                tokio::spawn(async move {
                    match client.create_session().await {
                        Ok(session) => {
                            sender.send(Ok(session)).ok();
                        }
                        Err(e) => {
                            sender.send(Err(e.to_string())).ok();
                        }
                    }
                });
            }
            PageAction::SendMessage {
                session_id,
                message,
                model,
            } => {
                let client = self.api_client.clone();
                tokio::spawn(async move {
                    let request = SendMessageRequest {
                        message_id: None,
                        model,
                        agent: None,
                        no_reply: None,
                        system: None,
                        tools: None,
                        parts: vec![PartInput::Text {
                            id: None,
                            text: message,
                            synthetic: None,
                            ignored: None,
                        }],
                    };
                    match client.send_message(&session_id, request).await {
                        Ok(_) => {
                            log::info!("Message sent to session {}", session_id);
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to send message to session {}: {}",
                                session_id,
                                e
                            );
                        }
                    }
                });
            }
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
            self.handle_action(action);
        }

        let mut page_ctx = PageContext {
            api_client: &self.api_client,
            action_sender: &self.action_sender,
            current_sessions: &self.current_sessions,
        };
        self.pages_router.mount(ctx, &mut page_ctx);
    }
}
