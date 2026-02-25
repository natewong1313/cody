use chrono::{DateTime, NaiveDateTime, Utc};
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::DatabaseError,
    harness::{
        Harness, ModelSelection, OpencodeMessage, OpencodeMessageWithParts, OpencodePart,
        OpencodePartInput, OpencodeSendMessageRequest,
    },
    proto_session,
    proto_utils::{format_naive_datetime, parse_naive_datetime, parse_uuid},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub show_in_gui: bool,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMessagePart {
    pub id: String,
    pub message_id: String,
    pub part_type: String,
    pub text: String,
    pub tool_json: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: Uuid,
    pub role: String,
    pub created_at: String,
    pub completed_at: String,
    pub parent_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub error_json: String,
    pub parts: Vec<SessionMessagePart>,
}

#[derive(Debug, Error)]
pub enum SessionRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("project not found for session.project_id {0}")]
    ProjectNotFound(Uuid),
    #[error("session not found for session.id {0}")]
    SessionNotFound(Uuid),
    #[error("harness id missing for session.id {0}")]
    HarnessIdMissing(Uuid),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("harness error: {0}")]
    Harness(String),
}

impl From<SessionRepoError> for tonic::Status {
    fn from(err: SessionRepoError) -> Self {
        match err {
            SessionRepoError::Database(e) => tonic::Status::internal(e.to_string()),
            SessionRepoError::ProjectNotFound(id) => {
                tonic::Status::not_found(format!("project not found: {id}"))
            }
            SessionRepoError::SessionNotFound(id) => {
                tonic::Status::not_found(format!("session not found: {id}"))
            }
            SessionRepoError::HarnessIdMissing(id) => {
                tonic::Status::failed_precondition(format!("session harness_id missing: {id}"))
            }
            SessionRepoError::InvalidInput(message) => tonic::Status::invalid_argument(message),
            SessionRepoError::Harness(message) => tonic::Status::unavailable(message),
        }
    }
}

impl From<Session> for proto_session::SessionModel {
    fn from(session: Session) -> Self {
        Self {
            id: session.id.to_string(),
            project_id: session.project_id.to_string(),
            show_in_gui: session.show_in_gui,
            name: session.name,
            created_at: format_naive_datetime(session.created_at),
            updated_at: format_naive_datetime(session.updated_at),
        }
    }
}

impl TryFrom<proto_session::SessionModel> for Session {
    type Error = Status;

    fn try_from(model: proto_session::SessionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: parse_uuid("session.id", &model.id)?,
            project_id: parse_uuid("session.project_id", &model.project_id)?,
            show_in_gui: model.show_in_gui,
            name: model.name,
            created_at: parse_naive_datetime("session.created_at", &model.created_at)?,
            updated_at: parse_naive_datetime("session.updated_at", &model.updated_at)?,
        })
    }
}

impl From<SessionMessagePart> for proto_session::SessionMessagePartModel {
    fn from(part: SessionMessagePart) -> Self {
        Self {
            id: part.id,
            message_id: part.message_id,
            r#type: part.part_type,
            text: part.text,
            tool_json: part.tool_json,
        }
    }
}

impl From<SessionMessage> for proto_session::SessionMessageModel {
    fn from(message: SessionMessage) -> Self {
        Self {
            id: message.id,
            session_id: message.session_id.to_string(),
            role: message.role,
            created_at: message.created_at,
            completed_at: message.completed_at,
            parent_id: message.parent_id,
            provider_id: message.provider_id,
            model_id: message.model_id,
            error_json: message.error_json,
            parts: message.parts.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto_session::SessionMessageInput> for OpencodeSendMessageRequest {
    type Error = Status;

    fn try_from(input: proto_session::SessionMessageInput) -> Result<Self, Self::Error> {
        if input.parts.is_empty() {
            return Err(Status::invalid_argument(
                "message input must include at least one part",
            ));
        }

        let mut parts = Vec::with_capacity(input.parts.len());
        for part in input.parts {
            if part.text.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "message part text cannot be empty for MVP text parts",
                ));
            }

            parts.push(OpencodePartInput::Text {
                id: None,
                text: part.text,
                synthetic: part.synthetic.then_some(true),
                ignored: part.ignored.then_some(true),
            });
        }

        let model = input.model.map(|model| ModelSelection {
            provider_id: model.provider_id,
            model_id: model.model_id,
        });

        Ok(Self {
            message_id: non_empty(input.message_id),
            model,
            agent: non_empty(input.agent),
            no_reply: input.no_reply.then_some(true),
            system: non_empty(input.system),
            tools: None,
            parts,
        })
    }
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn millis_to_datetime_string(ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|ts| format_naive_datetime(ts.naive_utc()))
        .unwrap_or_default()
}

fn map_part(part: OpencodePart) -> SessionMessagePart {
    match part {
        OpencodePart::Text(text) => SessionMessagePart {
            id: text.id,
            message_id: text.message_id,
            part_type: "text".to_string(),
            text: text.text,
            tool_json: String::new(),
        },
        OpencodePart::Reasoning(reasoning) => SessionMessagePart {
            id: reasoning.id,
            message_id: reasoning.message_id,
            part_type: "reasoning".to_string(),
            text: reasoning.text,
            tool_json: String::new(),
        },
        OpencodePart::Tool(tool) => SessionMessagePart {
            id: tool.id.clone(),
            message_id: tool.message_id.clone(),
            part_type: "tool".to_string(),
            text: String::new(),
            tool_json: serde_json::to_string(&tool).unwrap_or_default(),
        },
    }
}

fn map_message(session_id: Uuid, message: OpencodeMessageWithParts) -> SessionMessage {
    let id = message.id().to_string();
    let parts = message.parts.into_iter().map(map_part).collect();

    match message.info {
        OpencodeMessage::User(user) => SessionMessage {
            id,
            session_id,
            role: "user".to_string(),
            created_at: millis_to_datetime_string(user.time.created),
            completed_at: String::new(),
            parent_id: String::new(),
            provider_id: user.model.provider_id,
            model_id: user.model.model_id,
            error_json: String::new(),
            parts,
        },
        OpencodeMessage::Assistant(assistant) => SessionMessage {
            id,
            session_id,
            role: "assistant".to_string(),
            created_at: millis_to_datetime_string(assistant.time.created),
            completed_at: assistant
                .time
                .completed
                .map(millis_to_datetime_string)
                .unwrap_or_default(),
            parent_id: assistant.parent_id,
            provider_id: assistant.provider_id,
            model_id: assistant.model_id,
            error_json: assistant
                .error
                .and_then(|e| serde_json::to_string(&e).ok())
                .unwrap_or_default(),
            parts,
        },
    }
}

pub struct SessionRepo<D>
where
    D: crate::backend::db::Database,
{
    ctx: BackendContext<D>,
}

impl<D> SessionRepo<D>
where
    D: crate::backend::db::Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn list_by_project(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<Session>, SessionRepoError> {
        Ok(self.ctx.db.list_sessions_by_project(*project_id).await?)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Session>, SessionRepoError> {
        Ok(self.ctx.db.get_session(*id).await?)
    }

    pub async fn create(&self, session: &Session) -> Result<Session, SessionRepoError> {
        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        let project_dir = Some(project.dir.as_str());
        let harness_id = self
            .ctx
            .harness
            .create_session(session.clone(), project_dir)
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        let created = self.ctx.db.create_session(session.clone()).await?;
        if let Err(err) = self
            .ctx
            .db
            .set_session_harness_id(created.id, harness_id)
            .await
        {
            if let Err(delete_err) = self.ctx.db.delete_session(created.id).await {
                log::error!(
                    "failed to delete session {} after set_session_harness_id error: {}",
                    created.id,
                    delete_err
                );
            }
            return Err(SessionRepoError::Database(err));
        }

        Ok(created)
    }

    pub async fn update(&self, session: &Session) -> Result<Session, SessionRepoError> {
        Ok(self.ctx.db.update_session(session.clone()).await?)
    }

    pub async fn delete(&self, session_id: &Uuid) -> Result<(), SessionRepoError> {
        self.ctx.db.delete_session(*session_id).await?;
        Ok(())
    }

    pub async fn send_message(
        &self,
        session_id: &Uuid,
        input: proto_session::SessionMessageInput,
    ) -> Result<SessionMessage, SessionRepoError> {
        let session = self
            .ctx
            .db
            .get_session(*session_id)
            .await?
            .ok_or(SessionRepoError::SessionNotFound(*session_id))?;

        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        let harness_id = self
            .ctx
            .db
            .get_session_harness_id(*session_id)
            .await?
            .ok_or(SessionRepoError::HarnessIdMissing(*session_id))?;

        let request = OpencodeSendMessageRequest::try_from(input)
            .map_err(|e| SessionRepoError::InvalidInput(e.to_string()))?;

        let message = self
            .ctx
            .harness
            .send_message(&harness_id, &request, Some(project.dir.as_str()))
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        Ok(map_message(*session_id, message))
    }

    pub async fn get_messages(
        &self,
        session_id: &Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<SessionMessage>, SessionRepoError> {
        let session = self
            .ctx
            .db
            .get_session(*session_id)
            .await?
            .ok_or(SessionRepoError::SessionNotFound(*session_id))?;

        let project = self
            .ctx
            .db
            .get_project(session.project_id)
            .await?
            .ok_or(SessionRepoError::ProjectNotFound(session.project_id))?;

        let harness_id = self
            .ctx
            .db
            .get_session_harness_id(*session_id)
            .await?
            .ok_or(SessionRepoError::HarnessIdMissing(*session_id))?;

        let messages = self
            .ctx
            .harness
            .get_session_messages(&harness_id, limit, Some(project.dir.as_str()))
            .await
            .map_err(|e| SessionRepoError::Harness(e.to_string()))?;

        Ok(messages
            .into_iter()
            .map(|message| map_message(*session_id, message))
            .collect())
    }
}
