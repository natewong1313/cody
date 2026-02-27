use std::{pin::Pin, sync::Arc};

use futures::{Stream, stream};
use tonic::{Request, Response, Status};

use crate::backend::{
    BackendService, Message, MessageModel, MessagePart, MessagePartModel, MessageWithParts,
    SendMessageReply, SendMessageRequest,
    harness::{ModelSelection, OpencodePartInput, OpencodeSendMessageRequest},
    proto_message::{
        self, ListMessagesBySessionReply, ListMessagesBySessionRequest,
        SubscribeSessionMessagesReply, SubscribeSessionMessagesRequest,
        message_server::Message as MessageService, subscribe_session_messages_reply,
    },
    proto_utils::{self, parse_uuid},
    repo::message_events::MessageDiffEvent,
};

#[tonic::async_trait]
impl MessageService for Arc<BackendService> {
    type SubscribeSessionMessagesStream =
        Pin<Box<dyn Stream<Item = Result<SubscribeSessionMessagesReply, Status>> + Send + 'static>>;

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageReply>, Status> {
        let req = request.into_inner();
        let session_id = parse_uuid("session_id", &req.session_id)?;
        if req.text.trim().is_empty() {
            return Err(Status::invalid_argument("message text cannot be empty"));
        }

        let model = if !req.provider_id.is_empty() && !req.model_id.is_empty() {
            Some(ModelSelection {
                provider_id: req.provider_id,
                model_id: req.model_id,
            })
        } else {
            None
        };

        let harness_request = OpencodeSendMessageRequest {
            message_id: None,
            model,
            agent: (!req.agent.is_empty()).then_some(req.agent),
            no_reply: None,
            system: (!req.system.is_empty()).then_some(req.system),
            tools: None,
            parts: vec![OpencodePartInput::Text {
                id: None,
                text: req.text,
                synthetic: None,
                ignored: None,
            }],
        };

        let result = self
            .message_repo
            .send_user_message(&session_id, &harness_request)
            .await?;

        Ok(Response::new(SendMessageReply {
            session_id: session_id.to_string(),
            user_message_id: result.user_message.id.to_string(),
        }))
    }

    async fn list_messages_by_session(
        &self,
        request: Request<ListMessagesBySessionRequest>,
    ) -> Result<Response<ListMessagesBySessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        let messages = self.message_repo.list_by_session(&session_id).await?;

        let mut mapped = Vec::with_capacity(messages.len());
        for message in messages {
            let parts = self.message_part_repo.list_by_message(&message.id).await?;
            mapped.push(MessageWithParts {
                message: Some(map_message(message)),
                parts: parts.into_iter().map(map_message_part).collect(),
            });
        }

        Ok(Response::new(ListMessagesBySessionReply {
            messages: mapped,
        }))
    }

    async fn subscribe_session_messages(
        &self,
        request: Request<SubscribeSessionMessagesRequest>,
    ) -> Result<Response<Self::SubscribeSessionMessagesStream>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        let receiver = self.message_events_sender.subscribe();

        let updates = stream::unfold(
            (receiver, session_id),
            |(mut receiver, session_id)| async move {
                loop {
                    match receiver.recv().await {
                        Ok(event) => {
                            if let Some(reply) = map_diff_event(session_id, event) {
                                return Some((Ok(reply), (receiver, session_id)));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    }
                }
            },
        );

        Ok(Response::new(Box::pin(updates)))
    }
}

fn map_diff_event(
    session_id: uuid::Uuid,
    event: MessageDiffEvent,
) -> Option<SubscribeSessionMessagesReply> {
    match event {
        MessageDiffEvent::MessageUpserted {
            session_id: sid,
            message,
        } if sid == session_id => Some(SubscribeSessionMessagesReply {
            payload: Some(subscribe_session_messages_reply::Payload::MessageUpserted(
                proto_message::MessageUpserted {
                    message: Some(map_message(message)),
                },
            )),
        }),
        MessageDiffEvent::MessagePartUpserted {
            session_id: sid,
            part,
            delta,
        } if sid == session_id => Some(SubscribeSessionMessagesReply {
            payload: Some(
                subscribe_session_messages_reply::Payload::MessagePartUpserted(
                    proto_message::MessagePartUpserted {
                        part: Some(map_message_part(part)),
                        has_delta: delta.is_some(),
                        delta: delta.unwrap_or_default(),
                    },
                ),
            ),
        }),
        MessageDiffEvent::MessageRemoved {
            session_id: sid,
            harness_message_id,
        } if sid == session_id => Some(SubscribeSessionMessagesReply {
            payload: Some(subscribe_session_messages_reply::Payload::MessageRemoved(
                proto_message::MessageRemoved {
                    session_id: sid.to_string(),
                    harness_message_id,
                },
            )),
        }),
        MessageDiffEvent::SessionIdle { session_id: sid } if sid == session_id => {
            Some(SubscribeSessionMessagesReply {
                payload: Some(subscribe_session_messages_reply::Payload::SessionIdle(
                    proto_message::SessionIdle {
                        session_id: sid.to_string(),
                    },
                )),
            })
        }
        _ => None,
    }
}

fn map_message(message: Message) -> MessageModel {
    MessageModel {
        id: message.id.to_string(),
        harness_message_id: message.harness_message_id.unwrap_or_default(),
        session_id: message.session_id.to_string(),
        parent_message_id: message
            .parent_message_id
            .map(|v| v.to_string())
            .unwrap_or_default(),
        role: message.role,
        body: message.body.unwrap_or_default(),
        is_finished_streaming: message.is_finished_streaming,
        is_summary: message.is_summary,
        model_id: message.model_id,
        provider_id: message.provider_id,
        error_name: message.error_name.unwrap_or_default(),
        error_message: message.error_message.unwrap_or_default(),
        error_type: message.error_type.unwrap_or_default(),
        cwd: message.cwd.unwrap_or_default(),
        root: message.root.unwrap_or_default(),
        cost: message.cost.unwrap_or_default(),
        input_tokens: message.input_tokens.unwrap_or_default(),
        output_tokens: message.output_tokens.unwrap_or_default(),
        reasoning_tokens: message.reasoning_tokens.unwrap_or_default(),
        cached_read_tokens: message.cached_read_tokens.unwrap_or_default(),
        cached_write_tokens: message.cached_write_tokens.unwrap_or_default(),
        total_tokens: message.total_tokens.unwrap_or_default(),
        completed_at: message
            .completed_at
            .map(proto_utils::format_naive_datetime)
            .unwrap_or_default(),
        created_at: proto_utils::format_naive_datetime(message.created_at),
        updated_at: proto_utils::format_naive_datetime(message.updated_at),
    }
}

fn map_message_part(part: MessagePart) -> MessagePartModel {
    MessagePartModel {
        id: part.id.to_string(),
        harness_part_id: part.harness_part_id.unwrap_or_default(),
        session_id: part.session_id.to_string(),
        message_id: part.message_id.to_string(),
        position: part.position,
        part_type: part.part_type,
        text_content: part.text_content.unwrap_or_default(),
        synthetic: part.synthetic.unwrap_or(false),
        ignored: part.ignored.unwrap_or(false),
        call_id: part.call_id.unwrap_or_default(),
        tool_name: part.tool_name.unwrap_or_default(),
        tool_status: part.tool_status.unwrap_or_default(),
        tool_title: part.tool_title.unwrap_or_default(),
        tool_output_text: part.tool_output_text.unwrap_or_default(),
        tool_error_text: part.tool_error_text.unwrap_or_default(),
        created_at: proto_utils::format_naive_datetime(part.created_at),
        updated_at: proto_utils::format_naive_datetime(part.updated_at),
    }
}
