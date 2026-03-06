use crate::backend::repo::{
    session::Session, user_message::UserMessage, user_message_part::UserMessagePart,
};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use uuid::Uuid;

pub mod event_forwarder;
pub mod opencode;
mod opencode_client;
pub(crate) use opencode_client::{OpencodePartInput, OpencodeSendMessageRequest};

pub struct Model {
    pub provider_id: String,
    pub model_id: String,
}

pub struct UserMessageRequest {
    pub id: Uuid,
    pub session_id: Uuid,
    pub model_id: String,
    pub provider_id: String,
    // pub msg_parts: Vec<MessageParts>,
}

#[derive(thiserror::Error, Debug)]
pub enum HarnessError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("API request failed: {0}")]
    ApiRequest(#[from] anyhow::Error),

    #[error("API transport failed: {0}")]
    ApiTransport(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct HarnessMessage {
    pub id: String,
    pub session_id: String,
}

pub type HarnessAssistantEventStream =
    Pin<Box<dyn Stream<Item = Result<HarnessAssistantEvent, HarnessError>> + Send>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HarnessSessionStatus {
    Idle,
    Busy,
    Retry {
        attempt: i64,
        message: String,
        next: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HarnessAssistantEvent {
    SessionStatus {
        harness_session_id: String,
        status: HarnessSessionStatus,
    },
    MessageUpdated {
        harness_session_id: String,
        message_id: String,
        completed_at: Option<i64>,
        error: Option<String>,
    },
    MessagePartUpdated {
        harness_session_id: String,
        message_id: String,
        part_id: String,
        part_type: String,
        payload: serde_json::Value,
    },
    MessagePartDelta {
        harness_session_id: String,
        message_id: String,
        part_id: String,
        field: String,
        delta: String,
    },
    SessionError {
        harness_session_id: Option<String>,
        error: String,
    },
}

impl HarnessMessage {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

pub trait Harness: Sized {
    fn new() -> anyhow::Result<Self>;
    fn cleanup(&self);

    async fn create_session(
        &self,
        session: Session,
        directory: Option<&str>,
    ) -> anyhow::Result<String>;

    async fn send_message_async(
        &self,
        harness_session_id: String,
        message: UserMessage,
        message_parts: Vec<UserMessagePart>,
        directory: Option<String>,
    ) -> Result<(), HarnessError>;

    async fn get_session_messages(
        &self,
        session_id: &str,
        limit: Option<i32>,
        directory: Option<&str>,
    ) -> Result<Vec<HarnessMessage>, HarnessError>;

    async fn listen_assistant_events(
        &self,
        harness_session_id: String,
        directory: Option<String>,
    ) -> Result<HarnessAssistantEventStream, HarnessError>;

    async fn get_event_stream(
        &self,
    ) -> anyhow::Result<
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
    >;
}
