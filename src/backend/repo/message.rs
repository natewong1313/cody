use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, DatabaseError},
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
}
