use crate::backend::{Project, Session};
use crate::components::button::{ButtonSize, StyledButton};
use crate::pages::{PageAction, PageContext, Route};
use crate::sync_engine::Loadable;
use crate::theme::{
    BG_50, BG_500, BG_700, BG_800, BG_900, BG_950, FUCHSIA_500, RADIUS_MD, STROKE_WIDTH,
};
use egui::epaint::CornerRadiusF32;
use egui::{
    Align2, CentralPanel, Color32, CornerRadius, Frame, Label, RichText, Stroke, TextEdit, Ui, vec2,
};
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, Style};
use egui_flex::{Flex, item};
use egui_phosphor::regular;
use uuid::Uuid;

pub struct ProjectPage {
    project_id: Option<Uuid>,
    sessions: Vec<Session>,
    session_tab_ids: Vec<Uuid>,

    session_tabs_tree: DockState<Uuid>,

    prompt_input: String,
}
struct TabViewer<'a> {
    sessions: &'a [Session],
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Uuid;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.sessions
            .iter()
            .find(|session| session.id == *tab)
            .map(|session| {
                if session.name.trim().is_empty() {
                    "New Session".to_string()
                } else {
                    session.name.clone()
                }
            })
            .unwrap_or_else(|| tab.to_string())
            .into()
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
        let session_tabs_tree = DockState::new(vec![]);
        Self {
            project_id: None,
            sessions: Vec::new(),
            session_tab_ids: Vec::new(),
            session_tabs_tree,
            prompt_input: "".to_string(),
        }
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        page_ctx: &mut super::PageContext,
        project_id: Uuid,
    ) {
        if self.project_id != Some(project_id) {
            self.sessions.clear();
            self.session_tab_ids.clear();
            self.session_tabs_tree = DockState::new(vec![]);
        }
        self.project_id = Some(project_id);

        page_ctx.sync_engine.ensure_project_loaded(project_id);
        page_ctx
            .sync_engine
            .ensure_sessions_by_project_loaded(project_id);

        CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .fill(BG_900)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                page_ctx.sync_engine.poll(ui);

                let sessions_state = page_ctx.sync_engine.sessions_by_project_state(project_id);
                if let Loadable::Ready(sessions) = &sessions_state {
                    self.sync_sessions(sessions);
                }

                match page_ctx.sync_engine.project_state(project_id) {
                    Loadable::Ready(Some(project)) => {
                        self.render_project(ui, page_ctx, &project, &sessions_state);
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

    fn sync_sessions(&mut self, sessions: &[Session]) {
        self.sessions = sessions.to_vec();

        let next_tab_ids: Vec<Uuid> = sessions.iter().map(|session| session.id).collect();
        if self.session_tab_ids != next_tab_ids {
            self.session_tab_ids = next_tab_ids.clone();
            self.session_tabs_tree = DockState::new(next_tab_ids);
        }
    }

    fn render_project(
        &mut self,
        ui: &mut Ui,
        page_ctx: &mut PageContext,
        project: &Project,
        sessions_state: &Loadable<Vec<Session>>,
    ) {
        self.render_project_navbar(ui, page_ctx, project);
        ui.add_space(12.0);
        // self.render_session(ui);

        match sessions_state {
            Loadable::Idle | Loadable::Loading => {
                ui.label(RichText::new("Loading sessions...").color(BG_500));
            }
            Loadable::Error(error) => {
                ui.label(RichText::new(error).color(egui::Color32::RED));
            }
            Loadable::Ready(_) if self.sessions.is_empty() => {
                ui.label(RichText::new("No sessions yet").color(BG_500));
            }
            Loadable::Ready(_) => {
                let mut dock_style = Style::from_egui(ui.style().as_ref());
                let active_text_color = BG_50;
                let inactive_text_color = BG_500;
                let focused_bg_color = BG_800;

                dock_style.tab.spacing = 6.0;
                dock_style.tab_bar.bg_fill = BG_900;
                dock_style.tab_bar.height = 36.0;
                dock_style.tab_bar.hline_color = Color32::TRANSPARENT;
                dock_style.tab_bar.inner_margin = egui::Margin::symmetric(8, 4);

                dock_style.tab.active.bg_fill = focused_bg_color;
                dock_style.tab.active.text_color = active_text_color;
                dock_style.tab.active.outline_color = egui::Color32::TRANSPARENT;
                dock_style.tab.active.corner_radius = CornerRadiusF32::same(RADIUS_MD).into();
                dock_style.tab.active_with_kb_focus.bg_fill = FUCHSIA_500;
                dock_style.tab.active_with_kb_focus.text_color = active_text_color;
                dock_style.tab.active_with_kb_focus.outline_color = egui::Color32::TRANSPARENT;

                dock_style.tab.focused.bg_fill = focused_bg_color;
                dock_style.tab.focused.text_color = active_text_color;
                dock_style.tab.focused.outline_color = egui::Color32::TRANSPARENT;
                dock_style.tab.focused.corner_radius = CornerRadiusF32::same(RADIUS_MD).into();
                dock_style.tab.focused_with_kb_focus.bg_fill = FUCHSIA_500;
                dock_style.tab.focused_with_kb_focus.text_color = active_text_color;
                dock_style.tab.focused_with_kb_focus.outline_color = egui::Color32::TRANSPARENT;

                dock_style.tab.inactive.bg_fill = BG_900;
                dock_style.tab.inactive.text_color = inactive_text_color;
                dock_style.tab.inactive.outline_color = egui::Color32::TRANSPARENT;
                dock_style.tab.inactive.corner_radius = CornerRadiusF32::same(RADIUS_MD).into();
                dock_style.tab.hovered.bg_fill = focused_bg_color;
                dock_style.tab.hovered.text_color = inactive_text_color;
                dock_style.tab.hovered.outline_color = egui::Color32::TRANSPARENT;
                dock_style.tab.hovered.corner_radius = CornerRadiusF32::same(RADIUS_MD).into();
                dock_style.tab.inactive_with_kb_focus.bg_fill = BG_900;
                dock_style.tab.inactive_with_kb_focus.text_color = inactive_text_color;
                dock_style.tab.inactive_with_kb_focus.outline_color = egui::Color32::TRANSPARENT;

                dock_style.buttons.close_tab_color = inactive_text_color;
                dock_style.buttons.close_tab_active_color = active_text_color;
                dock_style.buttons.close_tab_bg_fill = BG_900;

                dock_style.buttons.close_all_tabs_color = inactive_text_color;
                dock_style.buttons.close_all_tabs_active_color = active_text_color;
                dock_style.buttons.close_all_tabs_bg_fill = BG_800;

                dock_style.buttons.collapse_tabs_color = inactive_text_color;
                dock_style.buttons.collapse_tabs_active_color = active_text_color;
                dock_style.buttons.collapse_tabs_bg_fill = BG_800;

                DockArea::new(&mut self.session_tabs_tree)
                    .style(dock_style)
                    .show_inside(
                        ui,
                        &mut TabViewer {
                            sessions: &self.sessions,
                        },
                    );
            }
        }
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

    fn render_session(&mut self, ui: &mut Ui) {
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
