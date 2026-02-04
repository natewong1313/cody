use crate::{
    opencode::{OpencodeApiClient, OpencodeSession},
    prompt_input::PromptInput,
};
use egui::CentralPanel;
use egui_inbox::UiInbox;

#[derive(Default)]
pub enum Page {
    #[default]
    Sessions,
    Session,
}

pub struct App {
    api_client: OpencodeApiClient,
    prompt_input: PromptInput,
    current_page: Page,
    sessions_page: SessionsPage,
    session_page: SessionPage,
    session_inbox: UiInbox<Result<OpencodeSession, String>>,
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
            prompt_input: PromptInput::default(),
            current_page: Page::default(),
            sessions_page: SessionsPage::default(),
            session_page: SessionPage::default(),
            session_inbox: UiInbox::new(),
        }
    }

    /// Conditionally route pages
    fn mount_router(&mut self, ctx: &egui::Context) -> Option<Action> {
        CentralPanel::default()
            .show(ctx, |ui| match self.current_page {
                Page::Sessions => self.sessions_page.show(ui),
                Page::Session => self.session_page.show(ui),
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process async results from inbox
        for result in self.session_inbox.read(ctx) {
            match result {
                Ok(session) => {
                    log::info!("created session: {:?}", session);
                    self.current_page = Page::Session;
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

#[derive(Default)]
pub struct SessionsPage {}

impl SessionsPage {
    fn show(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        ui.label("sessions");
        let btn = ui.button("New Session");
        if btn.clicked() {
            return Some(Action::CreateSession);
        }

        None
    }
}

#[derive(Default)]
pub struct SessionPage {}

impl SessionPage {
    fn show(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        ui.label("session");

        None
    }
}
