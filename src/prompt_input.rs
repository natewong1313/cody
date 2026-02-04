use std::fmt::Display;
use std::path::Path;

use egui::{
    Align2, Button, ComboBox, Id, Key, Modifiers, Popup, PopupCloseBehavior, ScrollArea, TextEdit,
    text::{CCursor, CCursorRange},
};
use egui_flex::{Flex, FlexAlignContent, item};

#[derive(PartialEq)]
pub enum Model {
    Opus,
    Sonnet,
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Opus => write!(f, "Opus 4.5"),
            Model::Sonnet => write!(f, "Sonnet 4.5"),
        }
    }
}

pub struct PromptInput {
    pub text_input: String,
    pub selected_model: Option<Model>,
}

impl Default for PromptInput {
    fn default() -> Self {
        Self {
            text_input: String::new(),
            selected_model: None,
        }
    }
}

impl PromptInput {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let frame_height = 90.0;
        ui.allocate_ui(egui::vec2(ui.available_width(), frame_height), |ui| {
            egui::Frame::new()
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    Flex::vertical()
                        .w_full()
                        .h_full()
                        .gap(egui::vec2(0.0, 4.0))
                        .show(ui, |flex| {
                            // Text input - use add_ui to get TextEditOutput for cursor control
                            flex.add_ui(
                                item().grow(1.0).align_self_content(Align2::LEFT_TOP),
                                |ui| {
                                    TextEdit::multiline(&mut self.text_input)
                                        .frame(false)
                                        .desired_rows(1)
                                        .desired_width(ui.available_width())
                                        .hint_text("Ask anything!")
                                        .show(ui)
                                },
                            );

                            flex.add_flex(
                                item(),
                                Flex::horizontal()
                                    .w_full()
                                    .align_content(FlexAlignContent::Center)
                                    .gap(egui::vec2(8.0, 0.0)),
                                |flex| {
                                    flex.add_ui(item(), |ui| self.show_model_combobox(ui));
                                    flex.add_ui(item().grow(1.0), |_ui| {});
                                    flex.add_ui(item(), |ui| self.show_send_button(ui));
                                },
                            );
                        });
                });
        });
    }

    fn show_model_combobox(&mut self, ui: &mut egui::Ui) {
        ComboBox::from_label("")
            .selected_text(
                self.selected_model
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "Select model".to_string()),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.selected_model,
                    Some(Model::Opus),
                    Model::Opus.to_string(),
                );
                ui.selectable_value(
                    &mut self.selected_model,
                    Some(Model::Sonnet),
                    Model::Sonnet.to_string(),
                );
            });
    }

    fn show_send_button(&mut self, ui: &mut egui::Ui) {
        let send_button = Button::new("send");
        if ui.add(send_button).clicked() && !self.text_input.trim().is_empty() {
            println!("Sending: {}", self.text_input);
            self.text_input.clear();
        }
    }
}
