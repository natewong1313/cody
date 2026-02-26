use poll_promise::Promise;
use tonic::{Request, transport::Channel};
use uuid::Uuid;

use crate::backend::proto_message::{
    MessageInput, MessagePartInput, SendMessageRequest, message_client::MessageClient,
};

pub fn send_message(
    backend_channel: Channel,
    session_id: Uuid,
    message: String,
) -> Promise<Result<(), String>> {
    Promise::spawn_async(async move {
        let message = message.trim();
        log::debug!("sending message: {}", message);
        if message.is_empty() {
            return Ok(());
        }

        log::debug!("sending message: {}", message);

        let mut message_client = MessageClient::new(backend_channel.clone());
        let request = SendMessageRequest {
            session_id: session_id.to_string(),
            input: Some(MessageInput {
                parts: vec![MessagePartInput {
                    text: message.to_string(),
                    synthetic: false,
                    ignored: false,
                }],
                message_id: "".to_string(),
                agent: "build".to_string(),
                no_reply: false,
                system: "".to_string(),
                model: None,
            }),
        };

        message_client
            .send_message(Request::new(request))
            .await
            .map_err(|e| format!("failed to send message: {e}"))?;

        Ok(())
    })
}
