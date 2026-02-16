use crate::backend::Project;
use crate::pages::{PageAction, Route};
use crate::sync_engine::Loadable;
use egui::{CentralPanel, Frame, RichText};
use uuid::Uuid;

pub struct ProjectPage {
    project_id: Option<Uuid>,
}

impl ProjectPage {
    pub fn new() -> Self {
        Self { project_id: None }
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        page_ctx: &mut super::PageContext,
        project_id: Uuid,
    ) {
        self.project_id = Some(project_id);
        page_ctx.sync_engine.ensure_project_loaded(project_id);

        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                page_ctx.sync_engine.poll(ui);

                if ui.button("Back to projects").clicked() {
                    page_ctx
                        .action_sender
                        .send(PageAction::Navigate(Route::Projects))
                        .ok();
                }

                ui.add_space(12.0);

                match page_ctx.sync_engine.project_state(project_id) {
                    Loadable::Idle | Loadable::Loading => {
                        ui.label("Loading project...");
                    }
                    Loadable::Error(error) => {
                        ui.label(RichText::new(error).color(egui::Color32::RED));
                    }
                    Loadable::Ready(Some(project)) => {
                        self.render_project_details(ui, &project);
                    }
                    Loadable::Ready(None) => {
                        ui.label("Project not found");
                    }
                }
            });
    }

    fn render_project_details(&self, ui: &mut egui::Ui, project: &Project) {
        ui.heading(&project.name);
        ui.add_space(8.0);
        ui.label(format!("Id: {}", project.id));
        ui.label(format!("Directory: {}", project.dir));
    }
}
