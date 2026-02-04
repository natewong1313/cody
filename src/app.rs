use std::collections::HashMap;

use crate::opencode::{OpencodeApiClient, OpencodeSession};
use egui::CentralPanel;
use egui_inbox::UiInbox;

#[derive(Default)]
pub enum Page {
    #[default]
    Sessions,
    Session(String),
}

#[derive(Default)]
struct PageStates {
    session: HashMap<String, SessionPageState>,
}

#[derive(Default)]
struct SessionPageState {}

pub struct App {
    api_client: OpencodeApiClient,
    current_page: Page,
    page_states: PageStates,

    session_inbox: UiInbox<Result<OpencodeSession, String>>,
    current_sessions: HashMap<String, OpencodeSession>,
}

// Callbacks from pages that require operations at the app level
pub enum Action {
    Navigate(Page),
    CreateSession,
}

impl App {
    pub fn new(api_client: OpencodeApiClient) -> Self {
        Self {
            api_client,
            current_page: Page::default(),
            session_inbox: UiInbox::new(),
            page_states: PageStates::default(),
            current_sessions: HashMap::new(),
        }
    }

    /// Conditionally route pages
    fn mount_router(&mut self, ctx: &egui::Context) -> Option<Action> {
        CentralPanel::default()
            .show(ctx, |ui| match &self.current_page {
                Page::Sessions => self.render_sessions_page(ui),
                Page::Session(id) => self.render_session_page(ui, id.to_string()),
            })
            .inner
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Navigate(page) => self.current_page = page,
            Action::CreateSession => {
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
        }
    }

    fn render_sessions_page(&self, ui: &mut egui::Ui) -> Option<Action> {
        ui.label("sessions");
        let btn = ui.button("New Session");
        if btn.clicked() {
            return Some(Action::CreateSession);
        }

        None
    }

    fn render_session_page(&mut self, ui: &mut egui::Ui, id: String) -> Option<Action> {
        // let state = self.page_states.session.entry(id.clone()).or_default();
        let session = self.current_sessions.get(&id).unwrap();
        ui.label(format!("{}", session.id));

        None
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process async results from inbox
        for result in self.session_inbox.read(ctx) {
            match result {
                Ok(session) => {
                    log::info!("created session: {:?}", session);
                    self.current_page = Page::Session(session.id.clone());
                    self.current_sessions.insert(session.id.clone(), session);
                }
                Err(e) => {
                    log::error!("Failed to create session: {}", e);
                }
            }
        }

        let action = self.mount_router(ctx);
        if let Some(action) = action {
            self.handle_action(action);
        }
    }
}
