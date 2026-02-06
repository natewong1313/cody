use egui::{Align2, Button, Color32, Frame, TextEdit, vec2};
use egui_flex::{Flex, item};

pub struct SessionPage {
    session_id: String,
    prompt_input: String,
}

impl SessionPage {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            prompt_input: "".to_string(),
        }
    }
}

impl super::Page for SessionPage {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &mut super::PageContext) {
        let session = ctx.current_sessions.get(&self.session_id).unwrap();

        // ui.label("Session");
        ui.label(session.title.clone().unwrap_or("Hello".to_string()));
        // ui.add(TextEdit::multiline(&mut self.prompt_input).desired_width(f32::INFINITY));
        Frame::new()
            .inner_margin(8.0)
            .corner_radius(10.0)
            .fill(Color32::from_rgb(23, 23, 23))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(38, 38, 38)))
            .show(ui, |ui| {
                Flex::vertical()
                    .w_full()
                    .h_full()
                    .gap(vec2(0.0, 4.0))
                    .show(ui, |flex| {
                        flex.add(
                            item().grow(1.0).align_self_content(Align2::LEFT_TOP),
                            TextEdit::multiline(&mut self.prompt_input).frame(false),
                        );
                        flex.add_flex(
                            item(),
                            Flex::horizontal()
                                .align_content(egui_flex::FlexAlignContent::Center)
                                .gap(vec2(8.0, 0.0)),
                            |flex| {
                                flex.add(
                                    item(),
                                    Button::new(egui::RichText::new("Send").color(Color32::WHITE))
                                        .fill(Color32::from_rgb(217, 70, 239))
                                        .corner_radius(8.0)
                                        .min_size(vec2(80.0, 36.0)),
                                )
                            },
                        )
                    })
            });
    }
}
