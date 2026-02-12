use crate::theme::{
    BG_50, BG_500, BG_700, BG_800, FUCHSIA_300, FUCHSIA_500, FUCHSIA_700, RADIUS_MD, STROKE_WIDTH,
};
use egui::{
    text::LayoutJob, vec2, Button, FontFamily, FontId, Response, RichText, Stroke, StrokeKind, Ui,
    Vec2, Widget,
};
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
    icon: Option<&'a str>,
}

impl<'a> StyledButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            size: ButtonSize::default(),
            variant: ButtonVariant::default(),
            icon: None,
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

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let prev_padding = ui.spacing().button_padding;
        ui.spacing_mut().button_padding = self.size.padding();

        let (fill, stroke, hover_stroke, text_color) = match self.variant {
            ButtonVariant::Primary => (
                FUCHSIA_500,
                Stroke::NONE,
                Stroke::new(STROKE_WIDTH, FUCHSIA_700),
                BG_50,
            ),
            ButtonVariant::Secondary => (
                BG_800,
                Stroke::new(STROKE_WIDTH, BG_700),
                Stroke::new(STROKE_WIDTH, BG_500),
                BG_50,
            ),
        };

        let font_size = ui.style().text_styles[&egui::TextStyle::Button].size;

        let button = match self.icon {
            Some(icon) => {
                let mut job = LayoutJob::default();
                job.append(
                    icon,
                    0.0,
                    egui::TextFormat {
                        font_id: FontId::new(font_size, FontFamily::Name("phosphor".into())),
                        color: text_color,
                        valign: egui::Align::Center,
                        ..Default::default()
                    },
                );
                job.append(
                    &format!(" {}", self.text),
                    0.0,
                    egui::TextFormat {
                        font_id: FontId::new(font_size, FontFamily::Proportional),
                        color: text_color,
                        valign: egui::Align::Center,
                        ..Default::default()
                    },
                );
                Button::new(job)
            }
            None => Button::new(RichText::new(self.text).color(text_color)),
        }
        .fill(fill)
        .stroke(stroke)
        .corner_radius(RADIUS_MD);

        let response = ui.add(button);

        if response.hovered() {
            let rect = response.rect;
            ui.painter()
                .rect_stroke(rect, RADIUS_MD, hover_stroke, StrokeKind::Outside);
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
