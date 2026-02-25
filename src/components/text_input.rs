use crate::theme::{BG_50, BG_500, BG_700, BG_800, RADIUS_MD, STROKE_WIDTH};
use egui::{
    Align, FontFamily, FontId, Frame, Layout, Margin, Response, Stroke, TextEdit, TextFormat,
    TextStyle, Ui, Widget, text::LayoutJob,
};

#[derive(Default, Clone, Copy)]
#[allow(dead_code)]
pub enum TextInputSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl TextInputSize {
    fn margin(self) -> Margin {
        match self {
            TextInputSize::Sm => Margin::symmetric(6, 5),
            TextInputSize::Md => Margin::symmetric(10, 8),
            TextInputSize::Lg => Margin::symmetric(12, 10),
        }
    }
}

pub struct StyledTextInput<'a> {
    text: &'a mut String,
    hint_text: String,
    desired_width: f32,
    size: TextInputSize,
    leading_icon: Option<&'a str>,
}

impl<'a> StyledTextInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: String::new(),
            desired_width: f32::INFINITY,
            size: TextInputSize::default(),
            leading_icon: None,
        }
    }

    pub fn hint_text(mut self, hint: impl Into<String>) -> Self {
        self.hint_text = hint.into();
        self
    }

    #[allow(dead_code)]
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = width;
        self
    }

    #[allow(dead_code)]
    pub fn size(mut self, size: TextInputSize) -> Self {
        self.size = size;
        self
    }

    #[allow(dead_code)]
    pub fn leading_icon(mut self, icon: &'a str) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui) -> StyledTextInputResponse {
        let margin = self.size.margin();

        let inner_response = Frame::new()
            .fill(BG_800)
            .inner_margin(margin)
            .corner_radius(RADIUS_MD)
            .stroke(Stroke::new(STROKE_WIDTH, BG_700))
            .show(ui, |ui| {
                ui.set_height(0.0); // Prevent vertical expansion
                ui.set_min_width(ui.available_width()); // Fill horizontal space
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    if let Some(icon) = self.leading_icon {
                        let font_size = ui.style().text_styles[&TextStyle::Body].size;
                        let mut job = LayoutJob::default();
                        job.append(
                            icon,
                            0.0,
                            TextFormat {
                                font_id: FontId::new(
                                    font_size,
                                    FontFamily::Name("phosphor".into()),
                                ),
                                color: BG_500,
                                valign: Align::Center,
                                ..Default::default()
                            },
                        );
                        ui.label(job);
                    }

                    ui.add(
                        TextEdit::singleline(self.text)
                            .frame(false)
                            .hint_text(self.hint_text)
                            .desired_width(self.desired_width)
                            .text_color(BG_50),
                    )
                })
                .inner
            });

        StyledTextInputResponse {
            inner: inner_response.inner,
            frame_rect: inner_response.response.rect,
        }
    }
}

pub struct StyledTextInputResponse {
    pub inner: Response,
    #[allow(dead_code)]
    pub frame_rect: egui::Rect,
}

impl StyledTextInputResponse {
    #[allow(dead_code)]
    pub fn frame_height(&self) -> f32 {
        self.frame_rect.height()
    }
}

impl<'a> Widget for StyledTextInput<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).inner
    }
}
