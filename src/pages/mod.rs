use crate::{
    opencode::{ModelSelection, OpencodeSession},
    pages::project::ProjectPage,
    pages::projects::ProjectsPage,
    query::QueryClient,
};
use std::{collections::HashMap, sync::mpsc::Sender};
mod project;
mod projects;

#[derive(Debug, Clone, Default)]
pub enum Route {
    #[default]
    Projects,
    Project {
        id: uuid::Uuid,
    },
}

pub enum PageAction {
    Navigate(Route),
    // CreateSession,
    // SendMessage {
    //     session_id: String,
    //     message: String,
    //     model: Option<ModelSelection>,
    // },
}

pub struct PageContext<'a> {
    pub action_sender: &'a Sender<PageAction>,
    pub query: &'a mut QueryClient,
}

pub struct PagesRouter {
    current_page: Route,
    projects_page: ProjectsPage,
    project_pages: HashMap<uuid::Uuid, ProjectPage>,
}

impl PagesRouter {
    pub fn new() -> Self {
        Self {
            current_page: Route::default(),
            projects_page: ProjectsPage::new(),
            project_pages: HashMap::new(),
        }
    }

    pub fn mount(&mut self, ctx: &egui::Context, page_ctx: &mut PageContext) {
        match self.current_page.clone() {
            Route::Projects => self.projects_page.render(ctx, page_ctx),
            Route::Project { id } => self.project_page(id).render(ctx, page_ctx, id),
        }
    }

    pub fn navigate(&mut self, page: Route) {
        self.current_page = page
    }

    fn project_page(&mut self, id: uuid::Uuid) -> &mut ProjectPage {
        self.project_pages
            .entry(id)
            .or_insert_with(ProjectPage::new)
    }
}
