use crate::theme::{BG_50, BG_500, BG_700, BG_800, RADIUS_MD, STROKE_WIDTH};
use egui::{
    text::LayoutJob, vec2, Align, Button, FontFamily, FontId, Response, Stroke, TextFormat,
    TextStyle, TextWrapMode, Ui, Widget,
};
use egui_inbox::UiInbox;

pub struct DirButton<'a> {
    value: &'a str,
    placeholder: &'a str,
    icon: &'a str,
    min_width: Option<f32>,
    inbox: &'a UiInbox<String>,
    on_dir_change: Option<Box<dyn FnOnce(String) + 'a>>,
}

impl<'a> DirButton<'a> {
    pub fn new(value: &'a str, inbox: &'a UiInbox<String>) -> Self {
        Self {
            value,
            placeholder: "Select directory",
            icon: egui_phosphor::regular::FOLDER,
            min_width: None,
            inbox,
            on_dir_change: None,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = icon;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn on_dir_change(mut self, f: impl FnOnce(String) + 'a) -> Self {
        self.on_dir_change = Some(Box::new(f));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let font_size = ui.style().text_styles[&TextStyle::Button].size;
        let has_value = !self.value.is_empty();

        let mut job = LayoutJob::default();
        job.append(
            self.icon,
            0.0,
            TextFormat {
                font_id: FontId::new(font_size, FontFamily::Name("phosphor".into())),
                color: BG_50,
                valign: Align::Center,
                ..Default::default()
            },
        );
        job.append(
            &format!(
                " {}",
                if has_value {
                    self.value
                } else {
                    self.placeholder
                }
            ),
            0.0,
            TextFormat {
                font_id: FontId::new(font_size, FontFamily::Proportional),
                color: if has_value { BG_50 } else { BG_500 },
                valign: Align::Center,
                ..Default::default()
            },
        );

        let width = self.min_width.unwrap_or(ui.available_width());
        let min_size = vec2(width, 0.0);
        job.wrap.max_width = width - 24.0; // account for button padding
        job.wrap.break_anywhere = true;
        job.wrap.max_rows = 1;
        ui.spacing_mut().button_padding = vec2(12.0, 10.0);
        let button = Button::new(job)
            .corner_radius(RADIUS_MD)
            .fill(BG_800)
            .stroke(Stroke::new(STROKE_WIDTH, BG_700))
            .wrap_mode(TextWrapMode::Truncate)
            .min_size(min_size);

        let response = ui.add(button);

        if let Some(on_dir_change) = self.on_dir_change {
            if let Some(dir) = self.inbox.read(ui).last() {
                on_dir_change(dir);
            }
        }

        if response.clicked() {
            let sender = self.inbox.sender();
            let start_dir = if has_value {
                self.value.to_string()
            } else {
                "/".to_string()
            };
            std::thread::spawn(move || {
                if let Some(s) = rfd::FileDialog::new()
                    .set_directory(start_dir)
                    .pick_folder()
                    .and_then(|dir| dir.to_str().map(|s| s.to_string()))
                {
                    sender.send(s).ok();
                }
            });
        }

        response
    }
}

impl<'a> Widget for DirButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
