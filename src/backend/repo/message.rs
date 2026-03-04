use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, DatabaseError},
    harness::Harness,
    repo::{assistant_message::AssistantMessage, user_message::UserMessage},
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
    #[error("harness session not found for {0}")]
    HarnessSessionNotFound(Uuid),
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
    ) -> Result<UserMessage, MessageRepoError> {
        let session = self
            .ctx
            .db
            .get_session(message.session_id)
            .await?
            .ok_or(MessageRepoError::SessionNotFound(message.session_id))?;

        let Some(harness_session_id) = session.harness_session_id else {
            return Err(MessageRepoError::HarnessSessionNotFound(message.session_id));
        };

        let created_message = self.ctx.db.create_user_message(message.clone()).await?;

        log::debug!("sending message to harness");
        let _ = self
            .ctx
            .harness
            .send_message(harness_session_id, message, session.dir)
            .await;
        log::debug!("sent message to harness");

        Ok(created_message)
    }
}
