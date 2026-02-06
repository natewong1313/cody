use super::PageAction;
use crate::opencode::{EventPayload, GlobalEvent, Message, MessageWithParts, Part};
use egui::{Align2, Button, Color32, Frame, Layout, ScrollArea, TextEdit, vec2};
use egui_flex::{Flex, item};
use egui_inbox::UiInbox;
use std::collections::HashMap;

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
    messages_loaded: bool,
    streaming_text: HashMap<String, String>,
}

impl SessionPage {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            prompt_input: "".to_string(),
            message_event_inbox: UiInbox::new(),
            messages: Vec::new(),
            streaming: false,
            messages_loaded: false,
            streaming_text: HashMap::new(),
        }
    }

    fn fetch_messages(&self, ctx: &super::PageContext) {
        let sender = self.message_event_inbox.sender();
        let client = ctx.api_client.clone();
        let session_id = self.session_id.clone();

        tokio::spawn(async move {
            match client.get_session_messages(&session_id).await {
                Ok(messages) => {
                    sender
                        .send(MessageEventResult::MessagesLoaded(messages))
                        .ok();
                }
                Err(e) => {
                    sender.send(MessageEventResult::Error(e.to_string())).ok();
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

            use futures::StreamExt;

            while let Some(event_result) = stream.next().await {
                match event_result {
                    Ok(event) => {
                        if let Ok(global_event) = serde_json::from_str::<GlobalEvent>(&event.data) {
                            match global_event.payload {
                                EventPayload::MessageUpdated { props } => {
                                    // Filter by session_id
                                    let msg_session_id = match &props.info {
                                        Message::User(u) => &u.session_id,
                                        Message::Assistant(a) => &a.session_id,
                                    };
                                    if msg_session_id == &session_id {
                                        // Create a MessageWithParts from the info
                                        let msg_with_parts = MessageWithParts {
                                            info: props.info,
                                            parts: Vec::new(),
                                        };
                                        sender
                                            .send(MessageEventResult::MessageUpdated(
                                                msg_with_parts,
                                            ))
                                            .ok();
                                    }
                                }
                                EventPayload::MessagePartUpdated { props } => {
                                    // Filter by session_id from the part
                                    let part_session_id = match &props.part {
                                        Part::Text(t) => &t.session_id,
                                        Part::Reasoning(r) => &r.session_id,
                                        Part::Tool(t) => &t.session_id,
                                    };
                                    if part_session_id == &session_id {
                                        let message_id = match &props.part {
                                            Part::Text(t) => t.message_id.clone(),
                                            Part::Reasoning(r) => r.message_id.clone(),
                                            Part::Tool(t) => t.message_id.clone(),
                                        };
                                        sender
                                            .send(MessageEventResult::MessagePartUpdated {
                                                message_id,
                                                part: props.part,
                                                delta: props.delta,
                                            })
                                            .ok();
                                    }
                                }
                                EventPayload::MessageRemoved { props } => {
                                    if props.session_id == session_id {
                                        sender
                                            .send(MessageEventResult::MessageRemoved {
                                                message_id: props.message_id,
                                            })
                                            .ok();
                                    }
                                }
                                EventPayload::SessionIdle { props } => {
                                    if props.session_id == session_id {
                                        sender.send(MessageEventResult::SessionIdle).ok();
                                    }
                                }
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
        let id = match &msg.info {
            Message::User(u) => &u.id,
            Message::Assistant(a) => &a.id,
        };

        if let Some(existing) = self.messages.iter_mut().find(|m| match &m.info {
            Message::User(u) => &u.id == id,
            Message::Assistant(a) => &a.id == id,
        }) {
            *existing = msg;
        } else {
            self.messages.push(msg);
        }
    }

    fn update_message_part(&mut self, message_id: String, _part: Part, delta: Option<String>) {
        // For now, just accumulate streaming text
        if let Some(delta_text) = delta {
            self.streaming_text
                .entry(message_id)
                .and_modify(|text| text.push_str(&delta_text))
                .or_insert(delta_text);
        }
    }

    fn get_message_display_text(&self, msg: &MessageWithParts) -> String {
        let mut text = String::new();

        // Add parts text
        for part in &msg.parts {
            match part {
                Part::Text(t) => text.push_str(&t.text),
                _ => {}
            }
        }

        // Add streaming text if any
        let msg_id = match &msg.info {
            Message::User(u) => &u.id,
            Message::Assistant(a) => &a.id,
        };
        if let Some(streaming) = self.streaming_text.get(msg_id) {
            text.push_str(streaming);
        }

        text
    }
}

impl super::Page for SessionPage {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &mut super::PageContext) {
        // Load messages on first render
        if !self.messages_loaded {
            self.messages_loaded = true;
            self.fetch_messages(ctx);
            self.start_event_stream(ctx);
        }

        // Process incoming events
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
                    self.messages.retain(|m| match &m.info {
                        Message::User(u) => u.id != message_id,
                        Message::Assistant(a) => a.id != message_id,
                    });
                }
                MessageEventResult::SessionIdle => {
                    self.streaming = false;
                }
                MessageEventResult::Error(e) => {
                    log::error!("Message event error: {}", e);
                }
            }
        }

        // Main layout
        ui.vertical(|ui| {
            // Messages area
            ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                let available_height = ui.available_height() - 100.0; // Reserve space for input
                ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        for msg in &self.messages {
                            let (_role, is_user) = match &msg.info {
                                Message::User(_) => ("user", true),
                                Message::Assistant(_) => ("assistant", false),
                            };

                            let text = self.get_message_display_text(msg);
                            if text.is_empty() {
                                continue;
                            }

                            ui.with_layout(
                                if is_user {
                                    Layout::top_down(egui::Align::Max)
                                } else {
                                    Layout::top_down(egui::Align::Min)
                                },
                                |ui| {
                                    let bg_color = if is_user {
                                        Color32::from_rgb(217, 70, 239)
                                    } else {
                                        Color32::from_rgb(50, 50, 50)
                                    };
                                    let text_color = if is_user {
                                        Color32::WHITE
                                    } else {
                                        Color32::LIGHT_GRAY
                                    };

                                    Frame::new()
                                        .inner_margin(8.0)
                                        .corner_radius(8.0)
                                        .fill(bg_color)
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(&text)
                                                    .color(text_color)
                                                    .size(14.0),
                                            );
                                        });
                                },
                            );
                            ui.add_space(8.0);
                        }
                    });
            });

            // Input area
            ui.add_space(8.0);

            let mut send_clicked = false;

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
                            flex.add(
                                item().grow(1.0).align_self_content(Align2::LEFT_TOP),
                                TextEdit::multiline(&mut self.prompt_input).frame(false),
                            );
                            flex.add_flex(
                                item(),
                                Flex::horizontal()
                                    .align_content(egui_flex::FlexAlignContent::Center)
                                    .gap(vec2(8.0, 0.0)),
                                |flex| {
                                    if self.streaming {
                                        flex.add(
                                            item(),
                                            egui::Label::new(
                                                egui::RichText::new("Thinking...")
                                                    .color(Color32::YELLOW),
                                            ),
                                        );
                                    }
                                    let btn = flex.add(
                                        item(),
                                        Button::new(
                                            egui::RichText::new("Send").color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(217, 70, 239))
                                        .corner_radius(8.0)
                                        .min_size(vec2(80.0, 36.0)),
                                    );
                                    if btn.clicked() {
                                        send_clicked = true;
                                    }
                                },
                            );
                        })
                });

            if send_clicked && !self.prompt_input.trim().is_empty() {
                let message = self.prompt_input.trim().to_string();
                self.prompt_input.clear();
                ctx.action_sender
                    .send(PageAction::SendMessage {
                        session_id: self.session_id.clone(),
                        message,
                    })
                    .ok();
                self.streaming = true;
            }
        });
    }
}
