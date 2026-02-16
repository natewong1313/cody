use crate::{
    opencode::{ModelSelection, OpencodeApiClient, OpencodeSession},
    pages::projects::ProjectsPage,
    sync_engine::SyncEngineClient,
};
use std::{collections::HashMap, sync::mpsc::Sender};
mod project;
mod projects;

#[derive(Default)]
pub enum PageType {
    #[default]
    Projects,
    // Sessions,
    // Session(String),
}

pub enum PageAction {
    Navigate(PageType),
    CreateSession,
    SendMessage {
        session_id: String,
        message: String,
        model: Option<ModelSelection>,
    },
}

pub struct PageContext<'a> {
    pub api_client: &'a OpencodeApiClient,
    pub sync_engine: &'a SyncEngineClient,
    pub action_sender: &'a Sender<PageAction>,
    pub current_sessions: &'a HashMap<String, OpencodeSession>,
}

pub struct PagesRouter {
    current_page: PageType,
    projects_page: ProjectsPage,
}

impl PagesRouter {
    pub fn new() -> Self {
        Self {
            current_page: PageType::default(),
            projects_page: ProjectsPage::new(),
        }
    }

    pub fn mount(&mut self, ctx: &egui::Context, page_ctx: &mut PageContext) {
        match &self.current_page {
            PageType::Projects => self.projects_page.render(ctx, page_ctx),
            // PageType::Sessions => self.sessions_page.render(ctx, page_ctx),
            // PageType::Session(session_id) => self
            //     .get_session_page(session_id.to_string())
            //     .render(ctx, page_ctx),
        }
    }

    pub fn navigate(&mut self, page: PageType) {
        self.current_page = page
    }

    // fn get_session_page(&mut self, session_id: String) -> &mut session::SessionPage {
    //     self.session_pages
    //         .entry(session_id.clone())
    //         .or_insert_with(|| session::SessionPage::new(session_id.clone()))
    // }
}
