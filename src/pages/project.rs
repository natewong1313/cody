use crate::backend::Project;
use crate::components::button::{ButtonSize, StyledButton};
use crate::pages::{PageAction, PageContext, Route};
use crate::sync_engine::Loadable;
use crate::theme::{
    BG_50, BG_500, BG_700, BG_800, BG_900, BG_950, FUCHSIA_500, RADIUS_MD, STROKE_WIDTH,
};
use egui::{
    Align2, Button, CentralPanel, Color32, Context, Frame, Label, RichText, Stroke, TextEdit, Ui,
    vec2,
};
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_flex::{Flex, item};
use egui_phosphor::regular;
use uuid::Uuid;

pub struct ProjectPage {
    project_id: Option<Uuid>,

    tree: DockState<String>,

    prompt_input: String,
}
struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> OnCloseResponse {
        println!("Closed tab: {_tab}");
        OnCloseResponse::Close
    }
}

impl ProjectPage {
    pub fn new() -> Self {
        let mut tree = DockState::new(vec!["tab1".to_owned(), "tab2".to_owned()]);

        // You can modify the tree before constructing the dock
        // let [a, b] =
        //     tree.main_surface_mut()
        //         .split_left(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
        // let [_, _] = tree
        //     .main_surface_mut()
        //     .split_below(a, 0.7, vec!["tab4".to_owned()]);
        // let [_, _] = tree
        //     .main_surface_mut()
        //     .split_below(b, 0.5, vec!["tab5".to_owned()]);
        Self {
            project_id: None,
            tree,
            prompt_input: "".to_string(),
        }
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
                    Loadable::Ready(Some(project)) => {
                        self.render_project(ctx, ui, page_ctx, &project);
                    }
                    Loadable::Ready(None) => {
                        ui.label("Project not found");
                    }
                    Loadable::Idle | Loadable::Loading => {
                        ui.label("Loading project...");
                    }
                    Loadable::Error(error) => {
                        ui.label(RichText::new(error).color(egui::Color32::RED));
                    }
                }
            });
    }

    fn render_project(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        page_ctx: &mut PageContext,
        project: &Project,
    ) {
        self.render_project_navbar(ui, page_ctx, project);
        self.render_session(ctx, ui);

        // DockArea::new(&mut self.tree).show_inside(ui, &mut TabViewer {});
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

    fn render_session(&mut self, ctx: &Context, ui: &mut Ui) {
        Frame::new()
            .inner_margin(8.0)
            .outer_margin(8.0)
            .corner_radius(RADIUS_MD)
            .fill(BG_800)
            .stroke(Stroke::new(STROKE_WIDTH, BG_700))
            .show(ui, |ui| {
                Flex::vertical()
                    .w_full()
                    .gap(vec2(0.0, 16.0))
                    .show(ui, |flex| {
                        flex.add(
                            item().align_self_content(Align2::LEFT_TOP),
                            TextEdit::multiline(&mut self.prompt_input)
                                .hint_text("Type anything")
                                .frame(false),
                        );
                        flex.add_flex(
                            item(),
                            Flex::horizontal()
                                .w_full()
                                .justify(egui_flex::FlexJustify::SpaceBetween)
                                .align_items(egui_flex::FlexAlign::Center),
                            |flex| {
                                let btn = flex.add(item(), StyledButton::new("Send"));
                                if btn.clicked() {
                                    println!("send");
                                }
                            },
                        );
                    })
            });
    }
}
