use std::fmt::Display;
use std::path::Path;

use egui::{
    Align2, Button, ComboBox, Id, Key, Modifiers, Popup, PopupCloseBehavior, ScrollArea, Stroke,
    TextEdit,
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
        let text_input_id = ui.make_persistent_id("prompt_input_text_input");
        let text_input_focused = ui.ctx().memory(|mem| mem.has_focus(text_input_id));

        let parent_stroke = if text_input_focused {
            Stroke::new(1.0, ui.visuals().selection.stroke.color)
        } else {
            Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
        };

        ui.allocate_ui(egui::vec2(ui.available_width(), frame_height), |ui| {
            egui::Frame::new()
                .stroke(parent_stroke)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    Flex::vertical()
                        .w_full()
                        .h_full()
                        .gap(egui::vec2(0.0, 4.0))
                        .show(ui, |flex| {
                            flex.add_ui(
                                item().grow(1.0).align_self_content(Align2::LEFT_TOP),
                                |ui| self.show_text_input(text_input_id, ui),
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

    fn show_text_input(&mut self, id: Id, ui: &mut egui::Ui) {
        TextEdit::multiline(&mut self.text_input)
            .id(id)
            .frame(false)
            .desired_rows(1)
            .desired_width(ui.available_width())
            .hint_text("Ask anything!")
            .show(ui);
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
