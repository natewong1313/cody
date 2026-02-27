use crate::backend::repo::session::Session;
use futures::Stream;
use std::pin::Pin;
use uuid::Uuid;

pub mod opencode;
mod opencode_client;
pub(crate) use opencode_client::{
    ModelSelection, OpencodeEventPayload, OpencodeGlobalEvent, OpencodeMessage,
    OpencodeMessageWithParts, OpencodePart, OpencodePartInput, OpencodeSendMessageRequest,
};

pub struct Model {
    pub provider_id: String,
    pub model_id: String,
}

pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub model: Option<Model>,
    pub agent: Option<String>,
    pub system_msg: Option<String>,
    // pub msg_parts: Vec<MessageParts>,
}

pub trait Harness: Sized {
    fn new() -> anyhow::Result<Self>;
    fn cleanup(&self);

    async fn create_session(
        &self,
        session: Session,
        directory: Option<&str>,
    ) -> anyhow::Result<String>;

    async fn send_message(
        &self,
        session_id: &str,
        request: &OpencodeSendMessageRequest,
        directory: Option<&str>,
    ) -> anyhow::Result<OpencodeMessageWithParts>;

    async fn get_session_messages(
        &self,
        session_id: &str,
        limit: Option<i32>,
        directory: Option<&str>,
    ) -> anyhow::Result<Vec<OpencodeMessageWithParts>>;

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
