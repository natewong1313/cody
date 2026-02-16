use crate::theme::{BG_50, BG_500, BG_700, BG_900, FUCHSIA_500, RADIUS_MD, STROKE_WIDTH};
use egui::{Frame, Label, Response, RichText, Sense, Stroke, TextWrapMode, Ui};

pub struct ProjectCard<'a> {
    name: &'a str,
    dir: &'a str,
    index: usize,
}

impl<'a> ProjectCard<'a> {
    pub fn new(name: &'a str, dir: &'a str, index: usize) -> Self {
        Self { name, dir, index }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.push_id(self.index, |ui| {
            let frame_response = Frame::new()
                .fill(BG_900)
                .stroke(Stroke::new(STROKE_WIDTH, BG_700))
                .corner_radius(RADIUS_MD)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical(|ui| {
                        let first_letter = self
                            .name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();

                        Frame::new()
                            .fill(FUCHSIA_500)
                            .corner_radius(RADIUS_MD)
                            .show(ui, |ui| {
                                ui.set_width(36.0);
                                ui.set_height(36.0);
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new(first_letter)
                                            .color(BG_50)
                                            .strong()
                                            .size(16.0),
                                    );
                                });
                            });

                        ui.add_space(8.0);

                        ui.add(
                            Label::new(RichText::new(self.name).color(BG_50).strong().size(16.0))
                                .wrap_mode(TextWrapMode::Truncate),
                        );

                        ui.add(
                            Label::new(RichText::new(self.dir).color(BG_500).size(12.0))
                                .wrap_mode(TextWrapMode::Truncate),
                        );
                    });
                })
                .response;

            ui.interact(
                frame_response.rect,
                ui.id().with("click_area"),
                Sense::click(),
            )
        })
        .inner
    }
}
