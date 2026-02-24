use crate::backend::{Project, Session};
use crate::components::button::{ButtonSize, StyledButton};
use crate::pages::{PageAction, PageContext, Route};
use crate::theme::{
    BG_50, BG_500, BG_700, BG_800, BG_900, BG_950, FUCHSIA_500, RADIUS_MD, STROKE_WIDTH,
};
use egui::epaint::CornerRadiusF32;
use egui::{
    Align2, CentralPanel, Color32, Frame, Id, Label, RichText, ScrollArea, Stroke, TextEdit,
    TopBottomPanel, Ui, vec2,
};
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, Style, TabAddAlign};
use egui_flex::{Flex, item};
use egui_phosphor::regular;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type SessionTabStateMap = HashMap<Uuid, SessionTabState>;

#[derive(Default)]
struct SessionTabState {
    prompt_input: String,
}

pub struct ProjectPage {
    project_id: Option<Uuid>,
    session_tab_ids: Vec<Uuid>,

    session_tabs_tree: DockState<Uuid>,
    sessions_states: SessionTabStateMap,
}
struct TabViewer<'a> {
    sessions_by_id: &'a HashMap<Uuid, &'a Session>,
    sessions_states: &'a mut SessionTabStateMap,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Uuid;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        Id::new(*tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.sessions_by_id
            .get(tab)
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
        let tab_state = self.sessions_states.entry(*tab).or_default();

        TopBottomPanel::bottom(Id::new(("bottom_panel", *tab)))
            .show_separator_line(false)
            .default_height(120.0)
            .show_inside(ui, |ui| {
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
                                    TextEdit::multiline(&mut tab_state.prompt_input)
                                        .hint_text("Type anything")
                                        .frame(false)
                                        .desired_rows(2),
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
                                            println!("send: {}", tab_state.prompt_input);
                                            tab_state.prompt_input.clear();
                                        }
                                    },
                                );
                            })
                    });
            });
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for _ in 1..=50 {
                    ui.label("messages");
                }
            });
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
        }
        self.project_id = Some(project_id);

        CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .fill(BG_900)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                match page_ctx.query.use_sessions_by_project(ui, project_id) {
                    crate::query::QueryState::Loading => {
                        ui.label(RichText::new("Loading sessions...").color(BG_500));
                    }
                    crate::query::QueryState::Error(error) => {
                        ui.label(RichText::new(error).color(egui::Color32::RED));
                    }
                    crate::query::QueryState::Data(sessions) if sessions.is_empty() => {
                        ui.label(RichText::new("No sessions yet").color(BG_500));
                    }
                    crate::query::QueryState::Data(sessions) => {
                        self.sync_session_tabs(&sessions);
                        self.render_sessions_dock(ui, &sessions);
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

    fn render_project(
        &mut self,
        ui: &mut Ui,
        page_ctx: &mut PageContext,
        project: &Project,
        // sessions_state: &Loadable<Vec<Session>>,
    ) {
        self.render_project_navbar(ui, page_ctx, project);
        ui.add_space(12.0);
        // self.render_session(ui);

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
        dock_style.buttons.add_tab_bg_fill = BG_800;

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
            .show_add_buttons(true)
            .show_inside(
                ui,
                &mut TabViewer {
                    sessions_by_id: &sessions_by_id,
                    sessions_states: &mut self.sessions_states,
                },
            );
    }
}
