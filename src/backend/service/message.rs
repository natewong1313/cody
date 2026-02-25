use futures::{Stream, StreamExt, stream};
use std::{collections::hash_map::Entry, pin::Pin, sync::Arc};
use tokio::sync::watch;
use tonic::{Request, Response, Status};

use super::required_field;
use crate::backend::{
    BackendService,
    proto_message::{
        ListSessionMessagesReply, ListSessionMessagesRequest, SendMessageReply, SendMessageRequest,
        SubscribeSessionMessagesReply, SubscribeSessionMessagesRequest,
        message_server::Message as MessageService,
    },
    proto_utils::parse_uuid,
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
        let input = required_field(req.input, "input")?;

        let message = self.message_repo.send_message(&session_id, input).await?;
        self.publish_session_messages(session_id).await;

        Ok(Response::new(SendMessageReply {
            message: Some(message.into()),
        }))
    }

    async fn list_session_messages(
        &self,
        request: Request<ListSessionMessagesRequest>,
    ) -> Result<Response<ListSessionMessagesReply>, Status> {
        let req = request.into_inner();
        let session_id = parse_uuid("session_id", &req.session_id)?;
        if let Some(limit) = req.limit
            && limit <= 0
        {
            return Err(Status::invalid_argument("limit must be greater than 0"));
        }

        let messages = self
            .message_repo
            .list_messages(&session_id, req.limit)
            .await?;

        Ok(Response::new(ListSessionMessagesReply {
            messages: messages.into_iter().map(Into::into).collect(),
        }))
    }

    async fn subscribe_session_messages(
        &self,
        request: Request<SubscribeSessionMessagesRequest>,
    ) -> Result<Response<Self::SubscribeSessionMessagesStream>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        let (sender, mut receiver) = {
            let mut senders = self
                .message_sender_by_session_id
                .lock()
                .map_err(|_| Status::internal("message sender lock poisoned"))?;

            match senders.entry(session_id) {
                Entry::Occupied(entry) => {
                    let sender = entry.get().clone();
                    let receiver = sender.subscribe();
                    (sender, receiver)
                }
                Entry::Vacant(entry) => {
                    let (sender, receiver) = watch::channel(Vec::new());
                    entry.insert(sender.clone());
                    (sender, receiver)
                }
            }
        };

        let messages = self.message_repo.list_messages(&session_id, None).await?;
        let initial_messages: Vec<_> = messages.into_iter().map(Into::into).collect();
        sender.send_replace(initial_messages.clone());
        receiver.borrow_and_update();

        let initial_reply = SubscribeSessionMessagesReply {
            messages: initial_messages,
        };
        let initial = stream::once(async move { Ok(initial_reply) });
        let updates = stream::unfold(receiver, |mut receiver| async move {
            if receiver.changed().await.is_err() {
                return None;
            }

            let reply = SubscribeSessionMessagesReply {
                messages: receiver.borrow_and_update().clone(),
            };

            Some((Ok(reply), receiver))
        });

        Ok(Response::new(Box::pin(initial.chain(updates))))
    }
}
