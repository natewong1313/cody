use crate::backend::Session;
use crate::components::button::StyledButton;
use crate::theme::{BG_700, BG_800, RADIUS_MD, STROKE_WIDTH};
use egui::{Align2, Frame, Id, ScrollArea, Stroke, TextEdit, TopBottomPanel, vec2};
use egui_dock::tab_viewer::OnCloseResponse;
use egui_flex::{Flex, item};
use std::collections::HashMap;
use uuid::Uuid;

pub type SessionTabStateMap = HashMap<Uuid, SessionTabState>;

#[derive(Default)]
pub struct SessionTabState {
    prompt_input: String,
}

/// A tab viewer is responsible for all session tabs within a project
pub struct TabViewer<'a> {
    sessions_by_id: &'a HashMap<Uuid, &'a Session>,
    sessions_states: &'a mut SessionTabStateMap,
}

impl<'a> TabViewer<'a> {
    pub fn new(
        sessions_by_id: &'a HashMap<Uuid, &'a Session>,
        sessions_states: &'a mut SessionTabStateMap,
    ) -> Self {
        Self {
            sessions_by_id,
            sessions_states,
        }
    }
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
                                        let btn = flex.add(
                                            item(),
                                            StyledButton::new("Send").id("send_button"),
                                        );
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
                // for _ in 1..=50 {
                //     ui.label("messages");
                // }
            });
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> OnCloseResponse {
        println!("Closed tab: {tab}");
        OnCloseResponse::Close
    }
}
