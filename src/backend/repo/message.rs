use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, DatabaseError},
    harness::Harness,
    proto_message,
    repo::{
        assistant_message::{AssistantMessage, AssistantMessagePart},
        user_message::UserMessage,
        user_message_part::UserMessagePart,
    },
};

pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
}

impl From<Message> for proto_message::MessageHistory {
    fn from(value: Message) -> Self {
        match value {
            Message::User(user) => Self {
                message: Some(proto_message::message_history::Message::UserMessage(
                    user.into(),
                )),
            },
            Message::Assistant(assistant) => Self {
                message: Some(proto_message::message_history::Message::AssistantMessage(
                    assistant.into(),
                )),
            },
        }
    }
}

pub fn join_user_message_parts(
    message: UserMessage,
    parts: Vec<UserMessagePart>,
) -> proto_message::UserMessageModel {
    let mut model: proto_message::UserMessageModel = message.into();
    model.parts = parts.into_iter().map(Into::into).collect();
    model
}

pub fn join_assistant_message_parts(
    message: AssistantMessage,
    parts: Vec<AssistantMessagePart>,
) -> proto_message::AssistantMessageModel {
    let mut model: proto_message::AssistantMessageModel = message.into();
    model.parts = parts.into_iter().map(Into::into).collect();
    model
}

#[derive(Debug, thiserror::Error)]
pub enum MessageRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("session not found for {0}")]
    SessionNotFound(Uuid),
    #[error("harness error: {0}")]
    Harness(#[from] crate::backend::harness::HarnessError),
}

pub struct MessageRepo<D>
where
    D: Database,
{
    ctx: BackendContext<D>,
}

impl<D> MessageRepo<D>
where
    D: Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn list_by_session(
        &self,
        session_id: &Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, MessageRepoError> {
        Ok(self
            .ctx
            .db
            .list_messages_by_session(*session_id, limit)
            .await?)
    }

    pub async fn create_user_message(
        &self,
        message: UserMessage,
        mut message_parts: Vec<UserMessagePart>,
    ) -> Result<UserMessage, MessageRepoError> {
        let session = self
            .ctx
            .db
            .get_session(message.session_id)
            .await?
            .ok_or(MessageRepoError::SessionNotFound(message.session_id))?;

        log::debug!("sending message to harness");
        let _ = self
            .ctx
            .harness
            .send_message_async(
                session.harness_session_id,
                message.clone(),
                message_parts.clone(),
                session.dir,
            )
            .await?;
        log::debug!("sent message to harness");

        let created_message = self.ctx.db.create_user_message(message.clone()).await?;

        for message_part in &mut message_parts {
            message_part.user_message_id = created_message.id;
            message_part.session_id = created_message.session_id;
            let _ = self
                .ctx
                .db
                .create_user_message_part(message_part.clone())
                .await?;
        }

        Ok(created_message)
    }

    pub async fn list_user_messages(
        &self,
        session_id: &Uuid,
        limit: u32,
    ) -> Result<Vec<UserMessage>, MessageRepoError> {
        Ok(self
            .ctx
            .db
            .list_user_messages_by_session(*session_id, limit)
            .await?)
    }
}
