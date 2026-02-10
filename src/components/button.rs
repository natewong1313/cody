use crate::theme::{
    BG_50, BG_500, BG_700, BG_800, FUCHSIA_300, FUCHSIA_500, RADIUS_MD, STROKE_WIDTH,
};
use egui::{Button, Response, RichText, Stroke, StrokeKind, Ui, Vec2, Widget, vec2};
use egui_flex::{FlexInstance, FlexItem, FlexWidget};

#[derive(Default, Clone, Copy)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    fn padding(self) -> Vec2 {
        match self {
            ButtonSize::Sm => vec2(12.0, 9.0),
            ButtonSize::Md => vec2(16.0, 10.0),
            ButtonSize::Lg => vec2(20.0, 14.0),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
}

pub struct StyledButton<'a> {
    text: &'a str,
    size: ButtonSize,
    variant: ButtonVariant,
}

impl<'a> StyledButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            size: ButtonSize::default(),
            variant: ButtonVariant::default(),
        }
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let prev_padding = ui.spacing().button_padding;
        ui.spacing_mut().button_padding = self.size.padding();

        let (fill, stroke, focus_stroke, text_color) = match self.variant {
            ButtonVariant::Primary => (
                FUCHSIA_500,
                Stroke::NONE,
                Stroke::new(STROKE_WIDTH, FUCHSIA_300),
                BG_50,
            ),
            ButtonVariant::Secondary => (
                BG_800,
                Stroke::new(STROKE_WIDTH, BG_700),
                Stroke::new(STROKE_WIDTH, BG_50),
                BG_50,
            ),
        };

        let button = Button::new(RichText::new(self.text).color(text_color))
            .fill(fill)
            .stroke(stroke)
            .corner_radius(RADIUS_MD);

        let response = ui.add(button);

        if response.has_focus() {
            let rect = response.rect;
            ui.painter()
                .rect_stroke(rect, RADIUS_MD, focus_stroke, StrokeKind::Outside);
        }

        ui.spacing_mut().button_padding = prev_padding;

        response
    }
}

impl<'a> Widget for StyledButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

impl<'a> FlexWidget for StyledButton<'a> {
    type Response = Response;

    fn flex_ui(self, item: FlexItem, instance: &mut FlexInstance) -> Self::Response {
        instance.add_widget(item, self).inner
    }
}
