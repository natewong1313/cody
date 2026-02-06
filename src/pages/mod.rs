use crate::opencode::{OpencodeApiClient, OpencodeSession};
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};
mod session;
mod sessions;

#[derive(Default)]
pub enum PageType {
    #[default]
    Sessions,
    Session(String),
}

pub enum PageAction {
    Navigate(PageType),
    CreateSession,
    SendMessage { session_id: String, message: String },
}

pub struct PageContext<'a> {
    pub api_client: &'a OpencodeApiClient,
    pub action_sender: &'a Sender<PageAction>,
    pub current_sessions: &'a HashMap<String, OpencodeSession>,
}

pub trait Page {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &mut PageContext);
}

pub struct PagesRouter {
    current_page: PageType,
    sessions_page: sessions::SessionsPage,
    session_pages: HashMap<String, session::SessionPage>,
}

impl PagesRouter {
    pub fn new() -> Self {
        Self {
            current_page: PageType::default(),
            sessions_page: sessions::SessionsPage::new(),
            session_pages: HashMap::new(),
        }
    }

    pub fn mount(&mut self, ui: &mut egui::Ui, ctx: &mut PageContext) {
        match &self.current_page {
            PageType::Sessions => self.sessions_page.render(ui, ctx),
            PageType::Session(session_id) => self
                .get_session_page(session_id.to_string())
                .render(ui, ctx),
        }
    }

    pub fn navigate(&mut self, page: PageType) {
        self.current_page = page
    }

    fn get_session_page(&mut self, session_id: String) -> &mut session::SessionPage {
        self.session_pages
            .entry(session_id.clone())
            .or_insert_with(|| session::SessionPage::new(session_id.clone()))
    }
}
