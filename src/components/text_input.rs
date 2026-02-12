use crate::theme::{BG_50, BG_700, BG_800, RADIUS_MD, STROKE_WIDTH};
use egui::{Frame, Margin, Response, Stroke, TextEdit, Ui, Widget};

#[derive(Default, Clone, Copy)]
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
}

impl<'a> StyledTextInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: String::new(),
            desired_width: f32::INFINITY,
            size: TextInputSize::default(),
        }
    }

    pub fn hint_text(mut self, hint: impl Into<String>) -> Self {
        self.hint_text = hint.into();
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = width;
        self
    }

    pub fn size(mut self, size: TextInputSize) -> Self {
        self.size = size;
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
                ui.add(
                    TextEdit::singleline(self.text)
                        .frame(false)
                        .hint_text(self.hint_text)
                        .desired_width(self.desired_width)
                        .text_color(BG_50),
                )
            });

        StyledTextInputResponse {
            inner: inner_response.inner,
            frame_rect: inner_response.response.rect,
        }
    }
}

pub struct StyledTextInputResponse {
    pub inner: Response,
    pub frame_rect: egui::Rect,
}

impl StyledTextInputResponse {
    pub fn frame_height(&self) -> f32 {
        self.frame_rect.height()
    }
}

impl<'a> Widget for StyledTextInput<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).inner
    }
}
