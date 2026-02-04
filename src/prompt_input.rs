use std::fmt::Display;
use std::path::Path;

use egui::{
    text::{CCursor, CCursorRange},
    Align2, Button, ComboBox, Id, Key, Modifiers, Popup, PopupCloseBehavior, ScrollArea, TextEdit,
};
use egui_flex::{item, Flex, FlexAlignContent};

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

/// Recursively get all files in a directory
fn get_files_recursive(dir: &Path, base: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip hidden files and common ignored directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
            }
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(base) {
                    files.push(relative.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                files.extend(get_files_recursive(&path, base));
            }
        }
    }
    files.sort();
    files
}

pub struct PromptInput {
    pub text_input: String,
    pub selected_model: Option<Model>,
    // File selector state
    file_selector_active: bool,
    file_selector_index: usize,
    cached_files: Vec<String>,
    at_position: Option<usize>,
    prev_text_len: usize,
    // Cursor position to set after file selection
    pending_cursor_pos: Option<usize>,
}

impl Default for PromptInput {
    fn default() -> Self {
        Self {
            text_input: String::new(),
            selected_model: None,
            file_selector_active: false,
            file_selector_index: 0,
            cached_files: Vec::new(),
            at_position: None,
            prev_text_len: 0,
            pending_cursor_pos: None,
        }
    }
}

impl PromptInput {
    /// Get the query text after the '@' character
    fn get_file_query(&self) -> String {
        if let Some(at_pos) = self.at_position {
            self.text_input.get(at_pos + 1..).unwrap_or("").to_string()
        } else {
            String::new()
        }
    }

    /// Insert selected file path, replacing '@query' with the file path
    fn select_file(&mut self, file: &str) {
        if let Some(at_pos) = self.at_position {
            // Replace from '@' to end of current query with the file path
            self.text_input.replace_range(at_pos.., file);
            self.text_input.push(' '); // Add space after file path
                                       // Set pending cursor position to end of inserted text
            self.pending_cursor_pos = Some(self.text_input.len());
        }
        self.close_file_selector();
    }

    /// Close the file selector and reset state
    fn close_file_selector(&mut self) {
        self.file_selector_active = false;
        self.file_selector_index = 0;
        self.at_position = None;
    }

    /// Refresh the cached file list from current working directory
    fn refresh_files(&mut self) {
        if let Ok(cwd) = std::env::current_dir() {
            self.cached_files = get_files_recursive(&cwd, &cwd);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let frame_height = 90.0;

        // Detect '@' character being typed
        let curr_len = self.text_input.len();
        if curr_len > self.prev_text_len {
            // Text was added - check if '@' was typed
            if let Some('@') = self.text_input.chars().last() {
                if !self.file_selector_active {
                    self.file_selector_active = true;
                    self.at_position = Some(curr_len - 1);
                    self.file_selector_index = 0;
                    self.refresh_files();
                }
            }
        }
        self.prev_text_len = curr_len;

        // Check if '@' was deleted (close popup)
        if let Some(at_pos) = self.at_position {
            if at_pos >= self.text_input.len() || self.text_input.chars().nth(at_pos) != Some('@') {
                self.close_file_selector();
            }
        }

        // Get filtered files for the popup
        let query = self.get_file_query();
        let filtered_files: Vec<String> = self
            .cached_files
            .iter()
            .filter(|f| f.to_lowercase().contains(&query.to_lowercase()))
            .take(10)
            .cloned()
            .collect();

        // Handle keyboard navigation when file selector is active
        let mut selected_file: Option<String> = None;
        if self.file_selector_active && !filtered_files.is_empty() {
            ui.input_mut(|input| {
                if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                    self.file_selector_index =
                        (self.file_selector_index + 1).min(filtered_files.len() - 1);
                }
                if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                    self.file_selector_index = self.file_selector_index.saturating_sub(1);
                }
                if input.consume_key(Modifiers::NONE, Key::Enter) {
                    if let Some(file) = filtered_files.get(self.file_selector_index) {
                        selected_file = Some(file.clone());
                    }
                }
                if input.consume_key(Modifiers::NONE, Key::Escape) {
                    self.file_selector_active = false;
                }
            });
        }

        // Apply selection if Enter was pressed
        if let Some(file) = selected_file {
            self.select_file(&file);
        }

        // Clamp index to valid range
        if !filtered_files.is_empty() {
            self.file_selector_index = self.file_selector_index.min(filtered_files.len() - 1);
        }

        ui.allocate_ui(egui::vec2(ui.available_width(), frame_height), |ui| {
            egui::Frame::new()
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    Flex::vertical()
                        .w_full()
                        .h_full()
                        .gap(egui::vec2(0.0, 4.0))
                        .show(ui, |flex| {
                            // Text input - use add_ui to get TextEditOutput for cursor control
                            let pending_cursor = self.pending_cursor_pos.take();
                            let text_edit_response = flex.add_ui(
                                item().grow(1.0).align_self_content(Align2::LEFT_TOP),
                                |ui| {
                                    let available_width = ui.available_width();
                                    let output = TextEdit::multiline(&mut self.text_input)
                                        .frame(false)
                                        .desired_rows(1)
                                        .desired_width(available_width)
                                        .hint_text("Ask anything!")
                                        .show(ui);

                                    // Apply pending cursor position if set
                                    if let Some(pos) = pending_cursor {
                                        let cursor = CCursor::new(pos);
                                        let mut state = output.state.clone();
                                        state
                                            .cursor
                                            .set_char_range(Some(CCursorRange::one(cursor)));
                                        state.store(ui.ctx(), output.response.id);
                                    }

                                    output.response
                                },
                            );

                            // Show file selector popup if active
                            if self.file_selector_active {
                                let popup_id = Id::new("file_selector_popup");

                                // Store state needed for the closure
                                let current_index = self.file_selector_index;
                                let mut clicked_file: Option<String> = None;

                                Popup::from_response(&text_edit_response.inner)
                                    .id(popup_id)
                                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                                    .show(|ui| {
                                        ui.set_min_width(300.0);
                                        ui.set_max_height(200.0);

                                        if filtered_files.is_empty() {
                                            ui.label("No files found");
                                        } else {
                                            ScrollArea::vertical().max_height(200.0).show(
                                                ui,
                                                |ui| {
                                                    for (i, file) in
                                                        filtered_files.iter().enumerate()
                                                    {
                                                        let is_selected = i == current_index;
                                                        let response =
                                                            ui.selectable_label(is_selected, file);
                                                        if response.clicked() {
                                                            clicked_file = Some(file.clone());
                                                        }
                                                        // Scroll to selected item
                                                        if is_selected {
                                                            response.scroll_to_me(None);
                                                        }
                                                    }
                                                },
                                            );
                                        }
                                    });

                                // Handle click selection outside the closure
                                if let Some(file) = clicked_file {
                                    self.select_file(&file);
                                }
                            }

                            flex.add_flex(
                                item(),
                                Flex::horizontal()
                                    .w_full()
                                    .align_content(FlexAlignContent::Center)
                                    .gap(egui::vec2(8.0, 0.0)),
                                |flex| {
                                    // Model combobox
                                    flex.add_ui(item(), |ui| {
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
                                    });
                                    // Push button to right
                                    flex.add_ui(item().grow(1.0), |_ui| {});
                                    // Send button
                                    flex.add_ui(item(), |ui| {
                                        let send_button = Button::new("send");
                                        if ui.add(send_button).clicked()
                                            && !self.text_input.trim().is_empty()
                                        {
                                            println!("Sending: {}", self.text_input);
                                            self.text_input.clear();
                                        }
                                    });
                                },
                            );
                        });
                });
        });
    }
}
