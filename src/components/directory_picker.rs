use crate::components::button::{ButtonSize, ButtonVariant, StyledButton};
use crate::components::text_input::StyledTextInput;
use crate::theme::{BG_50, BG_700, BG_800, BG_900, RADIUS_MD, STROKE_WIDTH};
use egui::{
    vec2, Align, Area, Button, Frame, Id, Key, Modifiers, Order, Response, RichText, ScrollArea,
    Stroke, Ui, Widget,
};
use egui_phosphor::regular;
use std::path::{Path, PathBuf};

pub struct DirectoryPickerResponse {
    inner: Response,
    path: String,
    changed: bool,
    browse_clicked: bool,
}

impl DirectoryPickerResponse {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn browse_clicked(&self) -> bool {
        self.browse_clicked
    }
}

pub struct DirectoryPicker<'a> {
    path: &'a mut String,
    hint_text: Option<String>,
    width: Option<f32>,
    id: Id,
}

impl<'a> DirectoryPicker<'a> {
    pub fn new(path: &'a mut String) -> Self {
        Self {
            path,
            hint_text: None,
            width: None,
            id: Id::new("directory_picker"),
        }
    }

    pub fn hint_text(mut self, text: impl Into<String>) -> Self {
        self.hint_text = Some(text.into());
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = id.into();
        self
    }

    pub fn show(self, ui: &mut Ui) -> DirectoryPickerResponse {
        let search_text_id = self.id.with("search_text");
        let suggestions_id = self.id.with("suggestions");
        let focused_index_id = self.id.with("focused_index");
        let last_search_id = self.id.with("last_search");
        let scroll_to_focused_id = self.id.with("scroll_to_focused");
        let changed_id = self.id.with("changed");
        let browse_clicked_id = self.id.with("browse_clicked");

        let search_text: String = ui
            .ctx()
            .data_mut(|d| d.get_temp(search_text_id))
            .unwrap_or_else(|| self.path.clone());

        let mut suggestions: Vec<String> = ui
            .ctx()
            .data_mut(|d| d.get_temp(suggestions_id))
            .unwrap_or_default();

        let mut focused_index: Option<usize> = ui.ctx().data_mut(|d| d.get_temp(focused_index_id));

        let last_search: String = ui
            .ctx()
            .data_mut(|d| d.get_temp(last_search_id))
            .unwrap_or_default();

        let mut scroll_to_focused: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp(scroll_to_focused_id))
            .unwrap_or(false);

        let mut browse_clicked = false;
        let mut changed = false;

        // Get the current search text from temp storage or use the path as default
        let mut current_search = ui
            .ctx()
            .data_mut(|d| d.get_temp(search_text_id))
            .unwrap_or_else(|| search_text.clone());

        // Isolate the horizontal layout to prevent it from affecting parent form layouts
        let response = ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let btn_width = 36.0;
                let spacing = ui.spacing().item_spacing.x;
                let input_width = self
                    .width
                    .unwrap_or_else(|| (available - btn_width - spacing).max(100.0));

                let mut input = StyledTextInput::new(&mut current_search);
                if let Some(ref hint) = self.hint_text {
                    input = input.hint_text(hint);
                }
                input = input.desired_width(input_width);

                let text_response = input.show(ui);

                let btn_height = text_response.frame_height();
                let icon_btn = StyledButton::new("")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .icon(regular::FOLDER_OPEN)
                    .explicit_size(vec2(btn_width, btn_height))
                    .show(ui);

                if icon_btn.clicked() {
                    browse_clicked = true;
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        let path_str = folder.display().to_string();
                        current_search = path_str.clone();
                        *self.path = path_str;
                        suggestions.clear();
                        focused_index = None;
                        changed = true;
                    }
                }

                text_response
            })
        });

        // Store the updated search text back to temp storage
        ui.ctx().data_mut(|d| {
            d.insert_temp(search_text_id, current_search.clone());
        });

        *self.path = expand_tilde(&current_search);

        if current_search != last_search {
            suggestions = compute_dir_suggestions(&current_search);
            ui.ctx().data_mut(|d| {
                d.insert_temp(suggestions_id, suggestions.clone());
            });
            ui.ctx().data_mut(|d| {
                d.insert_temp(last_search_id, current_search.clone());
            });
            focused_index = if suggestions.is_empty() {
                None
            } else {
                Some(0)
            };
            scroll_to_focused = true;
            changed = true;
        }

        let show_popup = !current_search.is_empty()
            && !suggestions.is_empty()
            && response.inner.inner.inner.has_focus();

        if show_popup {
            let popup_id = self.id.with("popup");
            let suggestion_count = suggestions.len();

            if suggestion_count == 0 {
                focused_index = None;
            } else if let Some(idx) = focused_index {
                if idx >= suggestion_count {
                    focused_index = Some(suggestion_count - 1);
                }
            }

            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                let current = focused_index.unwrap_or(0);
                focused_index = Some((current + 1).min(suggestion_count.saturating_sub(1)));
                scroll_to_focused = true;
            }
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                let current = focused_index.unwrap_or(0);
                focused_index = Some(current.saturating_sub(1));
                scroll_to_focused = true;
            }
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                if let Some(focused_idx) = focused_index {
                    if let Some(path) = suggestions.get(focused_idx).cloned() {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(search_text_id, path.clone());
                        });
                        *self.path = path;
                        suggestions.clear();
                        focused_index = None;
                        changed = true;
                    }
                }
            }
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                suggestions.clear();
                focused_index = None;
            }

            let anchor_rect = response.inner.inner.frame_rect;
            let popup_pos = anchor_rect.left_bottom() + vec2(0.0, 4.0);

            Area::new(popup_id)
                .fixed_pos(popup_pos)
                .order(Order::Foreground)
                .interactable(true)
                .show(ui.ctx(), |ui| {
                    Frame::new()
                        .fill(BG_900)
                        .stroke(Stroke::new(STROKE_WIDTH, BG_700))
                        .corner_radius(RADIUS_MD)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(anchor_rect.width());
                            ui.set_max_width(anchor_rect.width());

                            let item_height: f32 = 28.0;
                            let max_visible: usize = 6;
                            let list_height = (item_height * suggestion_count as f32)
                                .min(item_height * max_visible as f32);

                            ScrollArea::vertical()
                                .max_height(list_height)
                                .show(ui, |ui| {
                                    let focused = focused_index.unwrap_or(usize::MAX);

                                    for (idx, suggestion) in suggestions.iter().enumerate() {
                                        let is_focused = idx == focused;

                                        let styles = ui.style_mut();
                                        styles.visuals.widgets.inactive.weak_bg_fill =
                                            if is_focused { BG_700 } else { BG_900 };
                                        styles.visuals.widgets.hovered.weak_bg_fill = BG_700;
                                        styles.visuals.widgets.active.weak_bg_fill = BG_800;
                                        styles.spacing.button_padding = vec2(8.0, 4.0);

                                        let btn = ui.add_sized(
                                            [ui.available_width(), item_height],
                                            Button::new(
                                                RichText::new(suggestion).color(BG_50).size(12.0),
                                            )
                                            .corner_radius(RADIUS_MD),
                                        );

                                        if is_focused && scroll_to_focused {
                                            btn.scroll_to_me(Some(Align::Center));
                                        }

                                        if btn.clicked() {
                                            ui.ctx().data_mut(|d| {
                                                d.insert_temp(search_text_id, suggestion.clone());
                                            });
                                            *self.path = suggestion.clone();
                                            suggestions.clear();
                                            focused_index = None;
                                            changed = true;
                                            return;
                                        }
                                    }
                                });
                        });
                });

            scroll_to_focused = false;
        }

        ui.ctx().data_mut(|d| {
            d.insert_temp(focused_index_id, focused_index);
            d.insert_temp(scroll_to_focused_id, scroll_to_focused);
            d.insert_temp(changed_id, changed);
            d.insert_temp(browse_clicked_id, browse_clicked);
        });

        DirectoryPickerResponse {
            inner: response.inner.inner.inner,
            path: self.path.clone(),
            changed,
            browse_clicked,
        }
    }
}

fn expand_tilde(input: &str) -> String {
    if input == "~" {
        return home_dir()
            .map(|h| h.display().to_string())
            .unwrap_or_else(|| input.to_string());
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest).display().to_string();
        }
    }
    input.to_string()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn compute_dir_suggestions(input: &str) -> Vec<String> {
    if input.is_empty() {
        return Vec::new();
    }

    let expanded = expand_tilde(input);

    let resolved = if !input.starts_with('/') && !input.starts_with('~') {
        match home_dir() {
            Some(home) => home.join(&expanded).display().to_string(),
            None => return Vec::new(),
        }
    } else {
        expanded
    };

    let path = Path::new(&resolved);

    let (parent, prefix) =
        if resolved.ends_with(std::path::MAIN_SEPARATOR) || resolved.ends_with('/') {
            (path.to_path_buf(), String::new())
        } else {
            let parent = path.parent().unwrap_or(Path::new("/")).to_path_buf();
            let prefix = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (parent, prefix)
        };

    let prefix_lower = prefix.to_lowercase();

    let Ok(entries) = std::fs::read_dir(&parent) else {
        return Vec::new();
    };

    let mut suggestions: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() {
                return None;
            }
            let name = entry.file_name();
            let name_str = name.to_str()?;

            if name_str.starts_with('.') {
                return None;
            }

            if !prefix_lower.is_empty() && !name_str.to_lowercase().starts_with(&prefix_lower) {
                return None;
            }

            Some(entry.path().display().to_string())
        })
        .collect();

    suggestions.sort();
    suggestions.truncate(50);

    suggestions
}

impl<'a> Widget for DirectoryPicker<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).inner
    }
}
