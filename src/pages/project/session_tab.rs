use crate::backend::Session;
use crate::components::button::StyledButton;
use crate::theme::{BG_700, BG_800, RADIUS_MD, STROKE_WIDTH};
use egui::{vec2, Align2, Color32, Frame, Id, Stroke, TextEdit, TopBottomPanel};
use egui_dock::tab_viewer::OnCloseResponse;
use egui_flex::{item, Flex};
use std::collections::HashMap;
use uuid::Uuid;

pub type SessionTabStateMap = HashMap<Uuid, SessionTabState>;

#[derive(Default)]
pub struct SessionTabState {
    prompt_input: String,
    send_msg_error: Option<String>,
}

/// A tab viewer is responsible for all session tabs within a project
pub struct TabViewer<'sessions> {
    sessions_by_id: &'sessions HashMap<Uuid, &'sessions Session>,
    sessions_states: &'sessions mut SessionTabStateMap,
}

impl<'sessions> TabViewer<'sessions> {
    pub fn new(
        sessions_by_id: &'sessions HashMap<Uuid, &'sessions Session>,
        sessions_states: &'sessions mut SessionTabStateMap,
    ) -> Self {
        Self {
            sessions_by_id,
            sessions_states,
        }
    }
}

impl<'sessions> egui_dock::TabViewer for TabViewer<'sessions> {
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
        let session_id = *tab;
        let session_state = self.sessions_states.entry(session_id).or_default();

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
                                    TextEdit::multiline(&mut session_state.prompt_input)
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
                                            let prompt = session_state.prompt_input.trim().to_string();
                                            if prompt.is_empty() {
                                                session_state.send_msg_error =
                                                    Some("Message cannot be empty".to_string());
                                            } else {
                                                log::info!(
                                                    "send clicked for session {session_id} ({} chars)",
                                                    prompt.len()
                                                );
                                                session_state.send_msg_error = Some(
                                                    "Message sending is temporarily disabled"
                                                        .to_string(),
                                                );
                                                session_state.prompt_input.clear();
                                            }
                                        }

                                        if let Some(err) = &session_state.send_msg_error {
                                            flex.add(
                                                item(),
                                                egui::Label::new(
                                                    egui::RichText::new(err).color(Color32::RED),
                                                ),
                                            );
                                        }
                                    },
                                );
                            })
                    });
            });
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> OnCloseResponse {
        println!("Closed tab: {tab}");
        OnCloseResponse::Close
    }
}
