use crate::backend::{Project, Session};
use crate::components::button::{ButtonSize, ButtonVariant, StyledButton};
use crate::pages::{PageAction, PageContext, Route};
use crate::query::QueryState;
use crate::theme::{BG_50, BG_500, BG_800, BG_900, BG_950, FUCHSIA_500, RADIUS_MD};
mod session_tab;
use egui::epaint::CornerRadiusF32;
use egui::{vec2, CentralPanel, Color32, Frame, Label, RichText, Ui};
use egui_dock::{DockArea, DockState, Style, TabAddAlign};
use egui_flex::{item, Flex};
use egui_phosphor::regular;
use session_tab::{SessionTabStateMap, TabViewer};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct ProjectPage {
    project_id: Option<Uuid>,
    redirected_missing_project: bool,
    session_tab_ids: Vec<Uuid>,

    session_tabs_tree: DockState<Uuid>,
    sessions_states: SessionTabStateMap,
}

impl ProjectPage {
    pub fn new() -> Self {
        let session_tabs_tree = DockState::new(vec![]);
        Self {
            project_id: None,
            redirected_missing_project: false,
            session_tab_ids: Vec::new(),
            session_tabs_tree,
            sessions_states: HashMap::new(),
        }
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        page_ctx: &mut super::PageContext,
        project_id: Uuid,
    ) {
        if self.project_id != Some(project_id) {
            self.session_tab_ids.clear();
            self.session_tabs_tree = DockState::new(vec![]);
            self.sessions_states.clear();
            self.redirected_missing_project = false;
        }
        self.project_id = Some(project_id);

        CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .fill(BG_900)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| match page_ctx.query.use_project(ui, project_id) {
                QueryState::Loading => {
                    ui.label(RichText::new("Loading project...").color(BG_500));
                }
                QueryState::Error(error) => {
                    ui.label(RichText::new(error).color(egui::Color32::RED));
                }
                QueryState::Data(None) => {
                    if !self.redirected_missing_project {
                        page_ctx
                            .action_sender
                            .send(PageAction::Navigate(Route::Projects))
                            .ok();
                        self.redirected_missing_project = true;
                    }
                }
                QueryState::Data(Some(project)) => {
                    self.redirected_missing_project = false;
                    self.render_project(ui, page_ctx, &project);

                    match page_ctx.query.use_sessions_by_project(ui, project_id) {
                        QueryState::Loading => {
                            ui.label(RichText::new("Loading sessions...").color(BG_500));
                        }
                        QueryState::Error(error) => {
                            ui.label(RichText::new(error).color(egui::Color32::RED));
                        }
                        QueryState::Data(sessions) if sessions.is_empty() => {
                            ui.label(RichText::new("No sessions yet").color(BG_500));
                        }
                        QueryState::Data(sessions) => {
                            self.sync_session_tabs(&sessions);
                            self.render_sessions_dock(ui, &sessions);
                        }
                    }
                }
            });
    }

    fn sync_session_tabs(&mut self, sessions: &[Session]) {
        let next_tab_ids: Vec<Uuid> = sessions.iter().map(|session| session.id).collect();
        let next_set: HashSet<Uuid> = next_tab_ids.iter().copied().collect();

        self.sessions_states
            .retain(|session_id, _| next_set.contains(session_id));

        let current_tab_ids: Vec<Uuid> = self
            .session_tabs_tree
            .iter_all_tabs()
            .map(|(_, tab_id)| *tab_id)
            .collect();
        let current_set: HashSet<Uuid> = current_tab_ids.iter().copied().collect();

        let sets_differ = current_set != next_set;
        let order_differs = current_tab_ids != next_tab_ids;

        if sets_differ {
            self.session_tabs_tree
                .retain_tabs(|tab_id| next_set.contains(tab_id));

            for session_id in &next_tab_ids {
                if self.session_tabs_tree.find_tab(session_id).is_none() {
                    self.session_tabs_tree.push_to_focused_leaf(*session_id);
                }
            }
        } else if order_differs {
            self.session_tabs_tree = DockState::new(next_tab_ids.clone());
        }

        self.session_tab_ids = next_tab_ids;
    }

    fn render_project(&mut self, ui: &mut Ui, page_ctx: &mut PageContext, project: &Project) {
        self.render_project_navbar(ui, page_ctx, project);
        ui.add_space(12.0);

        // match sessions_state {
        //     Loadable::Idle | Loadable::Loading => {
        //         ui.label(RichText::new("Loading sessions...").color(BG_500));
        //     }
        //     Loadable::Error(error) => {
        //         ui.label(RichText::new(error).color(egui::Color32::RED));
        //     }
        //     Loadable::Ready(sessions) if sessions.is_empty() => {
        //         ui.label(RichText::new("No sessions yet").color(BG_500));
        //     }
        //     Loadable::Ready(sessions) => {
        //         self.render_sessions_dock(ui, sessions);
        //     }
        // }
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
                            .variant(ButtonVariant::Ghost)
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

    fn render_sessions_dock(&mut self, ui: &mut Ui, sessions: &[Session]) {
        let sessions_by_id: HashMap<Uuid, &Session> = sessions
            .iter()
            .map(|session| (session.id, session))
            .collect();

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

        dock_style.buttons.add_tab_align = TabAddAlign::Left;
        dock_style.buttons.add_tab_color = inactive_text_color;
        dock_style.buttons.add_tab_active_color = active_text_color;
        dock_style.buttons.add_tab_bg_fill = Color32::TRANSPARENT;

        dock_style.buttons.close_tab_color = inactive_text_color;
        dock_style.buttons.close_tab_active_color = active_text_color;
        dock_style.buttons.close_tab_bg_fill = Color32::TRANSPARENT;

        dock_style.buttons.close_all_tabs_color = inactive_text_color;
        dock_style.buttons.close_all_tabs_active_color = active_text_color;
        dock_style.buttons.close_all_tabs_bg_fill = Color32::TRANSPARENT;

        dock_style.buttons.collapse_tabs_color = inactive_text_color;
        dock_style.buttons.collapse_tabs_active_color = active_text_color;
        dock_style.buttons.collapse_tabs_bg_fill = Color32::TRANSPARENT;

        DockArea::new(&mut self.session_tabs_tree)
            .style(dock_style)
            .show_add_buttons(true)
            .show_inside(
                ui,
                &mut TabViewer::new(&sessions_by_id, &mut self.sessions_states),
            );
    }
}
