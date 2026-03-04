use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, DatabaseError},
    harness::Harness,
    repo::{
        assistant_message::AssistantMessage,
        user_message::{UserMessage, UserMessagePart},
    },
};

pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
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
}
