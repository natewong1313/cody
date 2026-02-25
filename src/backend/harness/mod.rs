use crate::backend::repo::session::Session;

pub mod opencode;
mod opencode_client;
pub(crate) use opencode_client::{
    ModelSelection, OpencodeMessage, OpencodeMessageWithParts, OpencodePart, OpencodePartInput,
    OpencodeSendMessageRequest,
};

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
        harness_id: &str,
        request: &OpencodeSendMessageRequest,
        directory: Option<&str>,
    ) -> anyhow::Result<OpencodeMessageWithParts>;

    async fn get_session_messages(
        &self,
        harness_id: &str,
        limit: Option<i32>,
        directory: Option<&str>,
    ) -> anyhow::Result<Vec<OpencodeMessageWithParts>>;
}
