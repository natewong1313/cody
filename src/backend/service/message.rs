use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt, stream};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::required_field;
use crate::backend::{
    BackendService,
    harness::{Harness, HarnessAssistantEvent},
    proto_message::{
        self, CreateUserMessageReply, CreateUserMessageRequest, ListMessagesBySessionReply,
        ListMessagesBySessionRequest, SubscribeMessagesBySessionReply,
        SubscribeMessagesBySessionRequest, messages_server::Messages as MessageService,
    },
    proto_utils::parse_uuid,
    repo::{
        assistant_message::{AssistantMessage, AssistantMessagePart},
        message::MessageRepoError,
        user_message::UserMessage,
        user_message_part::UserMessagePart,
    },
};

type SubscribeStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<SubscribeMessagesBySessionReply, Status>> + Send>>;

#[tonic::async_trait]
impl MessageService for Arc<BackendService> {
    type SubscribeMessagesBySessionStream = SubscribeStream;

    async fn list_messages_by_session(
        &self,
        request: Request<ListMessagesBySessionRequest>,
    ) -> Result<Response<ListMessagesBySessionReply>, Status> {
        let req = request.into_inner();
        let session_id = parse_uuid("session_id", &req.session_id)?;
        let limit = if req.limit <= 0 {
            100
        } else {
            req.limit as u32
        };

        let messages = self
            .message_repo
            .list_by_session(&session_id, limit)
            .await
            .map_err(message_repo_error_to_status)?;

        Ok(Response::new(ListMessagesBySessionReply {
            messages: messages.into_iter().map(Into::into).collect(),
        }))
    }

    async fn create_user_message(
        &self,
        request: Request<CreateUserMessageRequest>,
    ) -> Result<Response<CreateUserMessageReply>, Status> {
        let req = request.into_inner();
        let message_model = required_field(req.message, "message")?;
        let parts_model = req.parts;

        let message = UserMessage::try_from(message_model.clone())?;
        let parts = parts_model
            .into_iter()
            .map(UserMessagePart::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let created = self
            .message_repo
            .create_user_message(message, parts)
            .await
            .map_err(message_repo_error_to_status)?;

        let mut model: proto_message::UserMessageModel = created.into();
        model.parts = message_model.parts;

        Ok(Response::new(CreateUserMessageReply {
            message: Some(model),
        }))
    }

    async fn subscribe_messages_by_session(
        &self,
        request: Request<SubscribeMessagesBySessionRequest>,
    ) -> Result<Response<Self::SubscribeMessagesBySessionStream>, Status> {
        let req = request.into_inner();
        let session_id = parse_uuid("session_id", &req.session_id)?;

        let session = self
            .session_repo
            .get(&session_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("session not found"))?;

        let initial_messages = self
            .message_repo
            .list_by_session(&session_id, 100)
            .await
            .map_err(message_repo_error_to_status)?;

        let events = self
            .ctx
            .harness
            .listen_assistant_events(session.harness_session_id.clone(), session.dir.clone())
            .await
            .map_err(|e| Status::unavailable(e.to_string()))?;

        let (tx, rx) =
            tokio::sync::mpsc::channel::<Result<SubscribeMessagesBySessionReply, Status>>(32);

        tx.send(Ok(SubscribeMessagesBySessionReply {
            messages: initial_messages.into_iter().map(Into::into).collect(),
        }))
        .await
        .map_err(|_| Status::internal("subscriber closed"))?;

        let backend = Arc::clone(self);
        tokio::spawn(async move {
            tokio::pin!(events);
            while let Some(item) = events.next().await {
                let event = match item {
                    Ok(event) => event,
                    Err(err) => {
                        let _ = tx.send(Err(Status::unavailable(err.to_string()))).await;
                        break;
                    }
                };

                let changed = match apply_harness_event(&backend, session_id, event).await {
                    Ok(changed) => changed,
                    Err(err) => {
                        let _ = tx.send(Err(err)).await;
                        break;
                    }
                };

                if changed.is_empty() {
                    continue;
                }

                if tx
                    .send(Ok(SubscribeMessagesBySessionReply { messages: changed }))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        let output = stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|item| (item, rx))
        });

        Ok(Response::new(Box::pin(output)))
    }
}

async fn apply_harness_event(
    backend: &Arc<BackendService>,
    session_id: Uuid,
    event: HarnessAssistantEvent,
) -> Result<Vec<proto_message::MessageHistory>, Status> {
    match event {
        HarnessAssistantEvent::MessageUpdated {
            message_id,
            completed_at,
            error,
            ..
        } => {
            let changed =
                upsert_assistant_message(backend, session_id, &message_id, completed_at, error)
                    .await?;
            Ok(vec![proto_message::MessageHistory {
                message: Some(proto_message::message_history::Message::AssistantMessage(
                    changed.into(),
                )),
            }])
        }
        HarnessAssistantEvent::MessagePartUpdated {
            message_id,
            part_id,
            part_type,
            payload,
            ..
        } => {
            let assistant =
                upsert_assistant_message(backend, session_id, &message_id, None, None).await?;
            upsert_assistant_part(
                backend,
                session_id,
                assistant.id,
                &part_id,
                &part_type,
                Some(payload),
                None,
            )
            .await?;

            Ok(vec![proto_message::MessageHistory {
                message: Some(proto_message::message_history::Message::AssistantMessage(
                    assistant.into(),
                )),
            }])
        }
        HarnessAssistantEvent::MessagePartDelta {
            message_id,
            part_id,
            field,
            delta,
            ..
        } => {
            let assistant =
                upsert_assistant_message(backend, session_id, &message_id, None, None).await?;
            upsert_assistant_part(
                backend,
                session_id,
                assistant.id,
                &part_id,
                "text",
                None,
                Some((field, delta)),
            )
            .await?;

            Ok(vec![proto_message::MessageHistory {
                message: Some(proto_message::message_history::Message::AssistantMessage(
                    assistant.into(),
                )),
            }])
        }
        _ => Ok(Vec::new()),
    }
}

async fn upsert_assistant_message(
    backend: &Arc<BackendService>,
    session_id: Uuid,
    harness_message_id: &str,
    completed_at: Option<i64>,
    error: Option<String>,
) -> Result<AssistantMessage, Status> {
    let existing = backend
        .ctx
        .db
        .get_assistant_message_by_harness_id(session_id, harness_message_id.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut message = if let Some(message) = existing {
        message
    } else {
        AssistantMessage::new_from_harness(
            session_id,
            latest_user_message_id(backend, session_id).await?,
            harness_message_id,
        )
    };

    message.ensure_harness_message_id(harness_message_id);

    message.apply_harness_update(
        completed_at
            .and_then(|ms| DateTime::<Utc>::from_timestamp_millis(ms).map(|dt| dt.naive_utc())),
        error,
    );

    let saved = if backend
        .ctx
        .db
        .get_assistant_message(message.id)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .is_some()
    {
        backend
            .ctx
            .db
            .update_assistant_message(message)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
    } else {
        backend
            .ctx
            .db
            .create_assistant_message(message)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
    };

    Ok(saved)
}

async fn upsert_assistant_part(
    backend: &Arc<BackendService>,
    session_id: Uuid,
    assistant_message_id: Uuid,
    harness_part_id: &str,
    default_part_type: &str,
    payload: Option<serde_json::Value>,
    delta: Option<(String, String)>,
) -> Result<AssistantMessagePart, Status> {
    let existing = backend
        .ctx
        .db
        .get_assistant_message_part_by_harness_id(assistant_message_id, harness_part_id.to_string())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut part = existing.unwrap_or_else(|| {
        AssistantMessagePart::new_from_harness(
            session_id,
            assistant_message_id,
            harness_part_id,
            default_part_type,
        )
    });

    if let Some(payload) = payload {
        part.apply_payload_json(payload, default_part_type);
    }

    if let Some((field, value)) = delta {
        part.apply_delta(field, value);
    }

    part.ensure_harness_part_id(harness_part_id);

    let saved = if backend
        .ctx
        .db
        .get_assistant_message_part(part.id)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .is_some()
    {
        backend
            .ctx
            .db
            .update_assistant_message_part(part)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
    } else {
        backend
            .ctx
            .db
            .create_assistant_message_part(part)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
    };

    Ok(saved)
}

async fn latest_user_message_id(
    backend: &Arc<BackendService>,
    session_id: Uuid,
) -> Result<Uuid, Status> {
    let message_id = backend
        .message_repo
        .list_user_messages(&session_id, 1)
        .await
        .map_err(message_repo_error_to_status)?
        .into_iter()
        .next()
        .map(|m| m.id)
        .ok_or_else(|| Status::failed_precondition("no user message found"))?;
    Ok(message_id)
}

fn message_repo_error_to_status(err: MessageRepoError) -> Status {
    match err {
        MessageRepoError::Database(e) => Status::internal(e.to_string()),
        MessageRepoError::SessionNotFound(id) => {
            Status::not_found(format!("session not found: {id}"))
        }
        MessageRepoError::Harness(e) => Status::unavailable(e.to_string()),
    }
}
