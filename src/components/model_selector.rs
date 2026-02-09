use crate::theme::{BG_50, BG_500, BG_700, BG_800, BG_900, FUCHSIA_500, RADIUS_MD};
use egui::{
    Align, Button, Color32, FontSelection, Frame, InnerResponse, Key, Popup, PopupCloseBehavior,
    RectAlign, Response, RichText, ScrollArea, Style, TextEdit, text::LayoutJob, vec2,
};
use egui_flex::{FlexInstance, item};

#[derive(Debug, Clone)]
pub struct ModelOption {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub model_name: String,
    pub label: String,
}

pub struct ModelSelectorState {
    available_models: Vec<ModelOption>,
    selected_model_index: Option<usize>,
    focused_index: Option<usize>,

    search_text: String,
    last_search_text: String,
    filtered_models: Vec<(usize, ModelOption)>,

    popup_was_open: bool,
    scroll_to_focused: bool,
}

impl ModelSelectorState {
    pub fn new() -> Self {
        Self {
            available_models: Vec::new(),
            selected_model_index: None,
            search_text: String::new(),
            last_search_text: String::new(),
            filtered_models: Vec::new(),
            focused_index: None,
            popup_was_open: false,
            scroll_to_focused: false,
        }
    }

    pub fn set_models(&mut self, models: Vec<ModelOption>, default_index: Option<usize>) {
        self.available_models = models;
        self.selected_model_index = default_index;
        self.recompute_filtered_models();
    }

    fn recompute_filtered_models(&mut self) {
        let query = self.search_text.to_lowercase();
        self.filtered_models = self
            .available_models
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                query.is_empty()
                    || m.label.to_lowercase().contains(&query)
                    || m.model_name.to_lowercase().contains(&query)
                    || m.provider_name.to_lowercase().contains(&query)
            })
            .map(|(i, m)| (i, m.clone()))
            .collect();
    }

    pub fn selected_model(&self) -> Option<&ModelOption> {
        self.selected_model_index
            .and_then(|i| self.available_models.get(i))
    }
}

pub struct ModelSelector<'a> {
    state: &'a mut ModelSelectorState,
}

impl<'a> ModelSelector<'a> {
    pub fn new(state: &'a mut ModelSelectorState) -> Self {
        Self { state }
    }

    pub fn show(&mut self, trigger: &Response) {
        // Reset focused index when the button is clicked and popup wasn't already open
        if trigger.clicked() && !self.state.popup_was_open {
            self.state.focused_index = self.state.selected_model_index;
            self.state.scroll_to_focused = true;
        }

        let result = self.render_popup(trigger);
        self.state.popup_was_open = result.is_some();
    }

    fn render_popup(&mut self, trigger: &Response) -> Option<InnerResponse<()>> {
        // Recompute filtered models if search text has changed
        if self.state.search_text != self.state.last_search_text {
            self.state.recompute_filtered_models();
            self.state.last_search_text = self.state.search_text.clone();
        }

        let filtered = &self.state.filtered_models;
        let filtered_count = filtered.len();

        let result = Popup::from_toggle_button_response(trigger)
            .align(RectAlign::TOP_START)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .frame(Frame::popup(&Style::default()).fill(BG_900))
            .show(|ui| {
                ui.set_min_width(250.0);

                if self.state.available_models.is_empty() {
                    ui.label("No models available");
                    return;
                }

                // Clamp focused index to filtered range
                if filtered_count == 0 {
                    self.state.focused_index = None;
                } else if let Some(idx) = self.state.focused_index {
                    if idx >= filtered_count {
                        self.state.focused_index = Some(filtered_count - 1);
                    }
                }

                // Setup keyboard listeners
                self.setup_arrow_listeners(ui);

                // Render search bar
                self.render_search_bar(ui);

                // Render model list
                ui.add_space(4.0);
                self.render_model_list(ui);

                self.state.scroll_to_focused = false;
            });

        // Reset search when popup closes
        if result.is_none() && self.state.popup_was_open {
            self.state.search_text.clear();
            self.state.focused_index = None;
        }

        result
    }

    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        let prev_search = self.state.search_text.clone();
        let search_response = Frame::new()
            .fill(BG_800)
            .inner_margin(6.0)
            .corner_radius(RADIUS_MD)
            .stroke(egui::Stroke::new(1.0, BG_700))
            .show(ui, |ui| {
                ui.add(
                    TextEdit::singleline(&mut self.state.search_text)
                        .frame(false)
                        .hint_text("Search models...")
                        .desired_width(f32::INFINITY),
                )
            })
            .inner;

        if !self.state.popup_was_open {
            search_response.request_focus();
        }

        // Reset focused index when search text changes
        if self.state.search_text != prev_search {
            self.state.focused_index = if self.state.filtered_models.len() > 0 {
                Some(0)
            } else {
                None
            };
            self.state.scroll_to_focused = true;
        }
    }

    fn render_model_list(&mut self, ui: &mut egui::Ui) {
        let item_height: f32 = 40.0;
        let list_height: f32 = (item_height * 4.0).min(300.0);
        let filtered_count = self.state.filtered_models.len();

        if filtered_count == 0 {
            let (rect, _) = ui.allocate_exact_size(
                vec2(ui.available_width(), list_height),
                egui::Sense::hover(),
            );
            ui.put(rect, egui::Label::new("No matching models"));
        } else {
            ScrollArea::vertical()
                .min_scrolled_height(list_height)
                .max_height(list_height)
                .show(ui, |ui| {
                    let focused = self.state.focused_index.unwrap_or(usize::MAX);

                    for (filtered_idx, (real_idx, model)) in
                        self.state.filtered_models.iter().enumerate()
                    {
                        let is_focused = filtered_idx == focused;
                        let is_selected = self.state.selected_model_index == Some(*real_idx);

                        let response = self.render_model_item(ui, model, is_focused, is_selected);

                        if is_focused && self.state.scroll_to_focused {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }

                        if response.clicked() {
                            self.state.selected_model_index = Some(*real_idx);
                            ui.close();
                        }
                    }
                });
        }
    }

    fn render_model_item(
        &self,
        ui: &mut egui::Ui,
        model: &ModelOption,
        is_focused: bool,
        is_selected: bool,
    ) -> Response {
        let mut layout_job = LayoutJob::default();
        let style = Style::default();
        RichText::new(format!("{}\n", model.model_name))
            .color(BG_50)
            .append_to(&mut layout_job, &style, FontSelection::Default, Align::LEFT);
        RichText::new(&model.provider_name).color(BG_500).append_to(
            &mut layout_job,
            &style,
            FontSelection::Default,
            Align::LEFT,
        );

        let styles = ui.style_mut();
        // Use pink background when selected
        styles.visuals.widgets.inactive.weak_bg_fill = if is_selected {
            FUCHSIA_500
        } else if is_focused {
            BG_700
        } else {
            BG_900
        };
        styles.visuals.widgets.hovered.weak_bg_fill =
            if is_selected { FUCHSIA_500 } else { BG_700 };
        styles.visuals.widgets.active.weak_bg_fill = if is_selected { FUCHSIA_500 } else { BG_800 };
        styles.visuals.selection.bg_fill = if is_selected { FUCHSIA_500 } else { BG_800 };
        styles.spacing.button_padding = vec2(8.0, 4.0);

        ui.add_sized(
            [ui.available_width(), 0.0],
            egui::Button::new(layout_job)
                .corner_radius(RADIUS_MD)
                .right_text("")
                .selected(is_selected),
        )
    }

    /// Handles arrow navigation within the model selector
    fn setup_arrow_listeners(&mut self, ui: &mut egui::Ui) {
        let filtered_count = self.state.filtered_models.len();

        // Consume arrow/enter/escape keys before the text edit can eat them
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowDown)) {
            let current = self.state.focused_index.unwrap_or(0);
            self.state.focused_index = Some((current + 1).min(filtered_count.saturating_sub(1)));
            self.state.scroll_to_focused = true;
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowUp)) {
            let current = self.state.focused_index.unwrap_or(0);
            self.state.focused_index = Some(current.saturating_sub(1));
            self.state.scroll_to_focused = true;
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter)) {
            if let Some(focused_idx) = self.state.focused_index {
                if let Some(&(real_idx, _)) = self.state.filtered_models.get(focused_idx) {
                    self.state.selected_model_index = Some(real_idx);
                    ui.close();
                }
            }
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            ui.close();
        }
    }
}
