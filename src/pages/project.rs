use crate::backend::Project;
use crate::listen;
use egui::Ui;
use uuid::Uuid;

pub struct ProjectPage {
    project_id: Uuid,
    project: Option<Project>,
}

impl ProjectPage {
    pub fn new(project_id: Uuid) -> Self {
        Self {
            project_id,
            project: None,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, page_ctx: &mut super::PageContext) {}

    fn setup_listeners(&mut self, ui: &mut Ui, page_ctx: &mut super::PageContext) {
        listen!(
            self,
            ui,
            |ui| page_ctx.sync_engine.listen_project(ui, &self.project_id),
            project
        );
    }
}
