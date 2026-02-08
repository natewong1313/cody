use super::PageAction;
use crate::opencode::{EventPayload, GlobalEvent, MessageWithParts, ModelSelection, Part};
use egui::{Align2, Button, Color32, Frame, TextEdit, vec2};
use egui_dropdown::DropDownBox;
use egui_flex::{Flex, item};
use egui_inbox::UiInbox;
use futures::StreamExt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelOption {
    pub provider_id: String,
    pub model_id: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum ModelEventResult {
    ModelsLoaded {
        models: Vec<ModelOption>,
        default_index: Option<usize>,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub enum MessageEventResult {
    MessagesLoaded(Vec<MessageWithParts>),
    MessageUpdated(MessageWithParts),
    MessagePartUpdated {
        message_id: String,
        part: Part,
        delta: Option<String>,
    },
    MessageRemoved {
        message_id: String,
    },
    SessionIdle,
    Error(String),
}

pub struct SessionPage {
    session_id: String,
    prompt_input: String,
    message_event_inbox: UiInbox<MessageEventResult>,
    messages: Vec<MessageWithParts>,
    streaming: bool,
    first_render_occured: bool,
    streaming_text: HashMap<String, String>,
    // Model selector state
    model_inbox: UiInbox<ModelEventResult>,
    available_models: Vec<ModelOption>,
    selected_model_index: Option<usize>,
    model_search_buf: String,
}

impl SessionPage {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            prompt_input: "".to_string(),
            message_event_inbox: UiInbox::new(),
            messages: Vec::new(),
            streaming: false,
            first_render_occured: false,
            streaming_text: HashMap::new(),
            model_inbox: UiInbox::new(),
            available_models: Vec::new(),
            selected_model_index: None,
            model_search_buf: String::new(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, ctx: &mut super::PageContext) {
        if !self.first_render_occured {
            self.on_first_render(ctx);
        }
        self.process_events(ui);
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
                        self.render_textedit(flex);
                        self.render_action_bar(flex, ctx);
                    })
            });
    }

    fn on_first_render(&mut self, ctx: &mut super::PageContext) {
        self.first_render_occured = true;
        self.fetch_messages(ctx);
        self.fetch_models(ctx);
        self.start_event_stream(ctx);
    }

    fn process_events(&mut self, ui: &egui::Ui) {
        for event in self.model_inbox.read(ui.ctx()) {
            match event {
                ModelEventResult::ModelsLoaded {
                    models,
                    default_index,
                } => {
                    self.available_models = models;
                    self.selected_model_index = default_index;
                    // Set the search buffer to the default model's label
                    if let Some(idx) = default_index {
                        if let Some(model) = self.available_models.get(idx) {
                            self.model_search_buf = model.label.clone();
                        }
                    }
                }
                ModelEventResult::Error(e) => {
                    log::error!("Failed to fetch models: {}", e);
                }
            }
        }

        for event in self.message_event_inbox.read(ui.ctx()) {
            match event {
                MessageEventResult::MessagesLoaded(messages) => {
                    self.messages = messages;
                }
                MessageEventResult::MessageUpdated(msg) => {
                    self.upsert_message(msg);
                }
                MessageEventResult::MessagePartUpdated {
                    message_id,
                    part: _,
                    delta,
                } => {
                    self.streaming = true;
                    if let Some(delta_text) = delta {
                        self.streaming_text
                            .entry(message_id)
                            .and_modify(|text| text.push_str(&delta_text))
                            .or_insert(delta_text);
                    }
                }
                MessageEventResult::MessageRemoved { message_id } => {
                    self.messages.retain(|m| m.id() != message_id);
                }
                MessageEventResult::SessionIdle => {
                    self.streaming = false;
                }
                MessageEventResult::Error(e) => {
                    log::error!("Message event error: {}", e);
                }
            }
        }
    }

    fn render_textedit(&mut self, flex: &mut egui_flex::FlexInstance) {
        flex.add(
            item().grow(1.0).align_self_content(Align2::LEFT_TOP),
            TextEdit::multiline(&mut self.prompt_input).frame(false),
        );
    }

    fn render_action_bar(
        &mut self,
        flex: &mut egui_flex::FlexInstance,
        ctx: &mut super::PageContext,
    ) {
        flex.add_flex(
            item(),
            Flex::horizontal()
                .align_content(egui_flex::FlexAlignContent::Center)
                .gap(vec2(8.0, 0.0)),
            |flex| {
                // Model selector dropdown
                flex.add_ui(item(), |ui| {
                    let labels: Vec<String> = self
                        .available_models
                        .iter()
                        .map(|m| m.label.clone())
                        .collect();

                    let dropdown = DropDownBox::from_iter(
                        labels.iter().map(|s| s.as_str()),
                        "model_selector",
                        &mut self.model_search_buf,
                        |ui, text| ui.selectable_label(false, text),
                    )
                    .hint_text("Select model...")
                    .filter_by_input(true)
                    .select_on_focus(true)
                    .desired_width(250.0)
                    .max_height(300.0);

                    if ui.add(dropdown).changed() {
                        // Find the model matching the selected label
                        self.selected_model_index = self
                            .available_models
                            .iter()
                            .position(|m| m.label == self.model_search_buf);
                    }
                });

                if self.streaming {
                    flex.add(
                        item(),
                        egui::Label::new(egui::RichText::new("Thinking...").color(Color32::YELLOW)),
                    );
                }
                let btn = flex.add(
                    item(),
                    Button::new(egui::RichText::new("Send").color(Color32::WHITE))
                        .fill(Color32::from_rgb(217, 70, 239))
                        .corner_radius(8.0)
                        .min_size(vec2(80.0, 36.0)),
                );
                if btn.clicked() {
                    self.on_send_btn_clicked(ctx);
                }
            },
        );
    }

    fn on_send_btn_clicked(&mut self, ctx: &mut super::PageContext) {
        if self.prompt_input.trim().is_empty() {
            return;
        }
        let message = self.prompt_input.trim().to_string();
        self.prompt_input.clear();

        let model = self
            .selected_model_index
            .and_then(|i| self.available_models.get(i))
            .map(|m| ModelSelection {
                provider_id: m.provider_id.clone(),
                model_id: m.model_id.clone(),
            });

        ctx.action_sender
            .send(PageAction::SendMessage {
                session_id: self.session_id.clone(),
                message,
                model,
            })
            .ok();
        self.streaming = true;
    }

    fn fetch_messages(&self, ctx: &super::PageContext) {
        let sender = self.message_event_inbox.sender();
        let client = ctx.api_client.clone();
        let session_id = self.session_id.clone();

        tokio::spawn(async move {
            let result = client.get_session_messages(&session_id).await.map_or_else(
                |err| MessageEventResult::Error(err.to_string()),
                MessageEventResult::MessagesLoaded,
            );
            sender.send(result).unwrap();
        });
    }

    fn fetch_models(&self, ctx: &super::PageContext) {
        let sender = self.model_inbox.sender();
        let client = ctx.api_client.clone();

        tokio::spawn(async move {
            match client.get_providers().await {
                Ok(provider_response) => {
                    let connected: std::collections::HashSet<&str> = provider_response
                        .connected
                        .iter()
                        .map(|s| s.as_str())
                        .collect();

                    let mut models: Vec<ModelOption> = Vec::new();
                    for provider in &provider_response.all {
                        if !connected.contains(provider.id.as_str()) {
                            continue;
                        }
                        for model in provider.models.values() {
                            models.push(ModelOption {
                                provider_id: provider.id.clone(),
                                model_id: model.id.clone(),
                                label: format!("{} / {}", provider.name, model.name),
                            });
                        }
                    }
                    models.sort_by(|a, b| a.label.cmp(&b.label));

                    // Find the default model index using the "build" agent default
                    let default_index =
                        provider_response
                            .default
                            .get("build")
                            .and_then(|default_str| {
                                // default is in format "provider_id/model_id"
                                let parts: Vec<&str> = default_str.splitn(2, '/').collect();
                                if parts.len() == 2 {
                                    let (dprov, dmodel) = (parts[0], parts[1]);
                                    models.iter().position(|m| {
                                        m.provider_id == dprov && m.model_id == dmodel
                                    })
                                } else {
                                    None
                                }
                            });

                    sender
                        .send(ModelEventResult::ModelsLoaded {
                            models,
                            default_index,
                        })
                        .ok();
                }
                Err(e) => {
                    sender.send(ModelEventResult::Error(e.to_string())).ok();
                }
            }
        });
    }

    fn start_event_stream(&self, ctx: &super::PageContext) {
        let sender = self.message_event_inbox.sender();
        let client = ctx.api_client.clone();
        let session_id = self.session_id.clone();

        tokio::spawn(async move {
            let mut stream = match client.get_event_stream().await {
                Ok(stream) => stream,
                Err(e) => {
                    sender.send(MessageEventResult::Error(e.to_string())).ok();
                    return;
                }
            };

            while let Some(event_result) = stream.next().await {
                match event_result {
                    Ok(event) => {
                        if let Ok(global_event) = serde_json::from_str::<GlobalEvent>(&event.data) {
                            if let Some(result) = map_event_to_result(&global_event, &session_id) {
                                sender.send(result).ok();
                            }
                        }
                    }
                    Err(e) => {
                        sender.send(MessageEventResult::Error(e.to_string())).ok();
                    }
                }
            }
        });
    }

    fn upsert_message(&mut self, msg: MessageWithParts) {
        let id = msg.id().to_string();
        if let Some(existing) = self.messages.iter_mut().find(|m| m.id() == id) {
            *existing = msg;
        } else {
            self.messages.push(msg);
        }
    }
}

fn map_event_to_result(event: &GlobalEvent, session_id: &str) -> Option<MessageEventResult> {
    match &event.payload {
        EventPayload::MessageUpdated { props } if props.info.session_id() == session_id => {
            Some(MessageEventResult::MessageUpdated(MessageWithParts {
                info: props.info.clone(),
                parts: Vec::new(),
            }))
        }
        EventPayload::MessagePartUpdated { props } if props.part.session_id() == session_id => {
            Some(MessageEventResult::MessagePartUpdated {
                message_id: props.part.message_id().to_string(),
                part: props.part.clone(),
                delta: props.delta.clone(),
            })
        }
        EventPayload::MessageRemoved { props } if props.session_id == session_id => {
            Some(MessageEventResult::MessageRemoved {
                message_id: props.message_id.clone(),
            })
        }
        EventPayload::SessionIdle { props } if props.session_id == session_id => {
            Some(MessageEventResult::SessionIdle)
        }
        _ => None,
    }
}
