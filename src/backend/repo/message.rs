use chrono::{DateTime, Utc};
use futures::Stream;
use std::pin::Pin;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::DatabaseError,
    harness::{
        Harness, ModelSelection, OpencodeEventPayload, OpencodeGlobalEvent, OpencodeMessage,
        OpencodeMessageWithParts, OpencodePart, OpencodePartInput, OpencodeSendMessageRequest,
    },
    proto_message,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessagePart {
    pub id: String,
    pub message_id: String,
    pub part_type: String,
    pub text: String,
    pub tool_json: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: Uuid,
    pub role: String,
    pub created_at: String,
    pub completed_at: String,
    pub parent_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub error_json: String,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Error)]
pub enum MessageRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("session not found for session.id {0}")]
    SessionNotFound(Uuid),
    #[error("project not found for session.project_id {0}")]
    ProjectNotFound(Uuid),
    #[error("harness id missing for session.id {0}")]
    HarnessIdMissing(Uuid),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("harness error: {0}")]
    Harness(String),
}

impl From<MessageRepoError> for tonic::Status {
    fn from(err: MessageRepoError) -> Self {
        match err {
            MessageRepoError::Database(e) => tonic::Status::internal(e.to_string()),
            MessageRepoError::SessionNotFound(id) => {
                tonic::Status::not_found(format!("session not found: {id}"))
            }
            MessageRepoError::ProjectNotFound(id) => {
                tonic::Status::not_found(format!("project not found: {id}"))
            }
            MessageRepoError::HarnessIdMissing(id) => {
                tonic::Status::failed_precondition(format!("session harness_id missing: {id}"))
            }
            MessageRepoError::InvalidInput(message) => tonic::Status::invalid_argument(message),
            MessageRepoError::Harness(message) => tonic::Status::unavailable(message),
        }
    }
}

impl From<MessagePart> for proto_message::MessagePartModel {
    fn from(part: MessagePart) -> Self {
        Self {
            id: part.id,
            message_id: part.message_id,
            r#type: part.part_type,
            text: part.text,
            tool_json: part.tool_json,
        }
    }
}

impl From<Message> for proto_message::MessageModel {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            session_id: message.session_id.to_string(),
            role: message.role,
            created_at: message.created_at,
            completed_at: message.completed_at,
            parent_id: message.parent_id,
            provider_id: message.provider_id,
            model_id: message.model_id,
            error_json: message.error_json,
            parts: message.parts.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto_message::MessageInput> for OpencodeSendMessageRequest {
    type Error = Status;

    fn try_from(input: proto_message::MessageInput) -> Result<Self, Self::Error> {
        if input.parts.is_empty() {
            return Err(Status::invalid_argument(
                "message input must include at least one part",
            ));
        }

        let mut parts = Vec::with_capacity(input.parts.len());
        for part in input.parts {
            if part.text.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "message part text cannot be empty for MVP text parts",
                ));
            }

            parts.push(OpencodePartInput::Text {
                id: None,
                text: part.text,
                synthetic: part.synthetic.then_some(true),
                ignored: part.ignored.then_some(true),
            });
        }

        let model = input.model.map(|model| ModelSelection {
            provider_id: model.provider_id,
            model_id: model.model_id,
        });

        Ok(Self {
            message_id: non_empty(input.message_id),
            model,
            agent: non_empty(input.agent),
            no_reply: input.no_reply.then_some(true),
            system: non_empty(input.system),
            tools: None,
            parts,
        })
    }
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn millis_to_datetime_string(ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|ts| ts.naive_utc().to_string())
        .unwrap_or_default()
}

fn map_part(part: OpencodePart) -> MessagePart {
    match part {
        OpencodePart::Text(text) => MessagePart {
            id: text.id,
            message_id: text.message_id,
            part_type: "text".to_string(),
            text: text.text,
            tool_json: String::new(),
        },
        OpencodePart::Reasoning(reasoning) => MessagePart {
            id: reasoning.id,
            message_id: reasoning.message_id,
            part_type: "reasoning".to_string(),
            text: reasoning.text,
            tool_json: String::new(),
        },
        OpencodePart::Tool(tool) => MessagePart {
            id: tool.id.clone(),
            message_id: tool.message_id.clone(),
            part_type: "tool".to_string(),
            text: String::new(),
            tool_json: serde_json::to_string(&tool).unwrap_or_default(),
        },
    }
}

fn map_message(session_id: Uuid, message: OpencodeMessageWithParts) -> Message {
    let id = message.id().to_string();
    let parts = message.parts.into_iter().map(map_part).collect();

    match message.info {
        OpencodeMessage::User(user) => Message {
            id,
            session_id,
            role: "user".to_string(),
            created_at: millis_to_datetime_string(user.time.created),
            completed_at: String::new(),
            parent_id: String::new(),
            provider_id: user.model.provider_id,
            model_id: user.model.model_id,
            error_json: String::new(),
            parts,
        },
        OpencodeMessage::Assistant(assistant) => Message {
            id,
            session_id,
            role: "assistant".to_string(),
            created_at: millis_to_datetime_string(assistant.time.created),
            completed_at: assistant
                .time
                .completed
                .map(millis_to_datetime_string)
                .unwrap_or_default(),
            parent_id: assistant.parent_id,
            provider_id: assistant.provider_id,
            model_id: assistant.model_id,
            error_json: assistant
                .error
                .and_then(|e| serde_json::to_string(&e).ok())
                .unwrap_or_default(),
            parts,
        },
    }
}

pub struct MessageRepo<D>
where
    D: crate::backend::db::Database,
{
    ctx: BackendContext<D>,
}

impl<D> MessageRepo<D>
where
    D: crate::backend::db::Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn send_message(
        &self,
        session_id: &Uuid,
        input: proto_message::MessageInput,
    ) -> Result<Message, MessageRepoError> {
        let session = self
            .ctx
            .db
            .get_session(*session_id)
            .await?
            .ok_or(MessageRepoError::SessionNotFound(*session_id))?;

        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(MessageRepoError::ProjectNotFound(session.project_id))?;

        let harness_id = self
            .ctx
            .db
            .get_session_harness_id(*session_id)
            .await?
            .ok_or(MessageRepoError::HarnessIdMissing(*session_id))?;

        let request = OpencodeSendMessageRequest::try_from(input)
            .map_err(|e| MessageRepoError::InvalidInput(e.to_string()))?;

        let message = self
            .ctx
            .harness
            .send_message(&harness_id, &request, Some(project.dir.as_str()))
            .await
            .map_err(|e| MessageRepoError::Harness(e.to_string()))?;

        let mapped = map_message(*session_id, message);
        self.persist_message(&mapped).await?;

        Ok(mapped)
    }

    pub async fn list_messages(
        &self,
        session_id: &Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<Message>, MessageRepoError> {
        let session = self
            .ctx
            .db
            .get_session(*session_id)
            .await?
            .ok_or(MessageRepoError::SessionNotFound(*session_id))?;

        let _harness_id = self
            .ctx
            .db
            .get_session_harness_id(session.id)
            .await?
            .ok_or(MessageRepoError::HarnessIdMissing(*session_id))?;

        Ok(self
            .ctx
            .db
            .list_session_messages(*session_id, limit)
            .await?)
    }

    pub async fn get_event_stream(
        &self,
    ) -> Result<
        Pin<
            Box<
                dyn Stream<
                        Item = Result<
                            eventsource_stream::Event,
                            eventsource_stream::EventStreamError<reqwest::Error>,
                        >,
                    > + Send,
            >,
        >,
        MessageRepoError,
    > {
        self.ctx
            .harness
            .get_event_stream()
            .await
            .map_err(|e| MessageRepoError::Harness(e.to_string()))
    }

    pub async fn reconcile_session_messages(
        &self,
        session_id: &Uuid,
        limit: Option<i32>,
    ) -> Result<(), MessageRepoError> {
        let session = self
            .ctx
            .db
            .get_session(*session_id)
            .await?
            .ok_or(MessageRepoError::SessionNotFound(*session_id))?;

        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(MessageRepoError::ProjectNotFound(session.project_id))?;

        let harness_id = self
            .ctx
            .db
            .get_session_harness_id(*session_id)
            .await?
            .ok_or(MessageRepoError::HarnessIdMissing(*session_id))?;

        let messages = self
            .ctx
            .harness
            .get_session_messages(&harness_id, limit, Some(project.dir.as_str()))
            .await
            .map_err(|e| MessageRepoError::Harness(e.to_string()))?;

        for message in messages {
            let mapped = map_message(*session_id, message);
            self.persist_message(&mapped).await?;
        }

        Ok(())
    }

    async fn persist_message(&self, message: &Message) -> Result<(), MessageRepoError> {
        self.ctx
            .db
            .upsert_session_message_with_parts(message.clone())
            .await?;
        Ok(())
    }

    pub async fn ingest_event(
        &self,
        event: OpencodeGlobalEvent,
    ) -> Result<Option<Uuid>, MessageRepoError> {
        let harness_session_id = match &event.payload {
            OpencodeEventPayload::MessageUpdated { props } => props.info.session_id().to_string(),
            OpencodeEventPayload::MessagePartUpdated { props } => {
                props.part.session_id().to_string()
            }
            OpencodeEventPayload::MessageRemoved { props } => props.session_id.clone(),
            OpencodeEventPayload::SessionIdle { props } => props.session_id.clone(),
        };

        let Some(session_id) = self
            .ctx
            .db
            .get_session_id_by_harness_id(&harness_session_id)
            .await?
        else {
            return Ok(None);
        };

        match event.payload {
            OpencodeEventPayload::MessageUpdated { props } => {
                let info = props.info;
                let message = Message {
                    id: info.id().to_string(),
                    session_id,
                    role: match &info {
                        OpencodeMessage::User(_) => "user".to_string(),
                        OpencodeMessage::Assistant(_) => "assistant".to_string(),
                    },
                    created_at: match &info {
                        OpencodeMessage::User(user) => millis_to_datetime_string(user.time.created),
                        OpencodeMessage::Assistant(assistant) => {
                            millis_to_datetime_string(assistant.time.created)
                        }
                    },
                    completed_at: match &info {
                        OpencodeMessage::User(_) => String::new(),
                        OpencodeMessage::Assistant(assistant) => assistant
                            .time
                            .completed
                            .map(millis_to_datetime_string)
                            .unwrap_or_default(),
                    },
                    parent_id: match &info {
                        OpencodeMessage::User(_) => String::new(),
                        OpencodeMessage::Assistant(assistant) => assistant.parent_id.clone(),
                    },
                    provider_id: match &info {
                        OpencodeMessage::User(user) => user.model.provider_id.clone(),
                        OpencodeMessage::Assistant(assistant) => assistant.provider_id.clone(),
                    },
                    model_id: match &info {
                        OpencodeMessage::User(user) => user.model.model_id.clone(),
                        OpencodeMessage::Assistant(assistant) => assistant.model_id.clone(),
                    },
                    error_json: match &info {
                        OpencodeMessage::User(_) => String::new(),
                        OpencodeMessage::Assistant(assistant) => assistant
                            .error
                            .as_ref()
                            .and_then(|e| serde_json::to_string(e).ok())
                            .unwrap_or_default(),
                    },
                    parts: Vec::new(),
                };
                self.ctx.db.upsert_session_message(message).await?;
            }
            OpencodeEventPayload::MessagePartUpdated { props } => {
                let part = map_part(props.part);
                self.ctx
                    .db
                    .ensure_session_message_exists(session_id, &part.message_id)
                    .await?;
                self.ctx
                    .db
                    .upsert_session_message_part(session_id, part, props.delta)
                    .await?;
            }
            OpencodeEventPayload::MessageRemoved { props } => {
                self.ctx
                    .db
                    .mark_session_message_removed(session_id, &props.message_id)
                    .await?;
            }
            OpencodeEventPayload::SessionIdle { .. } => {
                self.reconcile_session_messages(&session_id, None).await?;
                return Ok(Some(session_id));
            }
        }

        Ok(Some(session_id))
    }
}
