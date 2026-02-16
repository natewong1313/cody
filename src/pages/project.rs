use crate::backend::Project;
use crate::components::button::{ButtonSize, StyledButton};
use crate::pages::{PageAction, PageContext, Route};
use crate::sync_engine::Loadable;
use crate::theme::{BG_50, BG_500, BG_900, BG_950};
use egui::{CentralPanel, Frame, Label, RichText, Ui, vec2};
use egui_flex::{Flex, item};
use egui_phosphor::regular;
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
            .frame(
                Frame::central_panel(&ctx.style())
                    .fill(BG_900)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                page_ctx.sync_engine.poll(ui);

                match page_ctx.sync_engine.project_state(project_id) {
                    Loadable::Idle | Loadable::Loading => {
                        ui.label("Loading project...");
                    }
                    Loadable::Error(error) => {
                        ui.label(RichText::new(error).color(egui::Color32::RED));
                    }
                    Loadable::Ready(Some(project)) => {
                        self.render_project_details(ui, page_ctx, &project);
                    }
                    Loadable::Ready(None) => {
                        ui.label("Project not found");
                    }
                }
            });
    }

    fn render_project_details(&self, ui: &mut Ui, page_ctx: &mut PageContext, project: &Project) {
        self.render_project_navbar(ui, page_ctx, project);
    }

    fn render_project_navbar(&self, ui: &mut Ui, page_ctx: &mut PageContext, project: &Project) {
        Frame::new().fill(BG_950).inner_margin(8.0).show(ui, |ui| {
            ui.set_width(ui.available_width());

            Flex::horizontal()
                .w_full()
                .gap(vec2(8.0, 0.0))
                .show(ui, |flex| {
                    flex.add(
                        item(),
                        StyledButton::new("")
                            .size(ButtonSize::Icon)
                            .icon_size(15.0)
                            .variant(crate::components::button::ButtonVariant::Ghost)
                            .icon(regular::SIDEBAR_SIMPLE),
                    );

                    let projects_label = flex.add(
                        item(),
                        Label::new(RichText::new("Projects").size(14.0).color(BG_500)),
                    );
                    if projects_label.clicked() {
                        page_ctx
                            .action_sender
                            .send(PageAction::Navigate(Route::Projects))
                            .ok();
                    }
                    flex.add(item(), Label::new(RichText::new("/").color(BG_500)));
                    flex.add(
                        item(),
                        Label::new(RichText::new(&project.name).size(14.0).color(BG_50)),
                    );
                });
        });
    }
}
