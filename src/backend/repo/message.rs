use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    BackendContext, MessagePart,
    db::{Database, DatabaseError},
    harness::{Harness, OpencodeMessageWithParts, OpencodePartInput, OpencodeSendMessageRequest},
    repo::message_events::MessageEventApplier,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub harness_message_id: Option<String>,
    pub session_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub role: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub agent: Option<String>,
    pub system_message: Option<String>,
    pub variant: Option<String>,
    pub is_finished_streaming: bool,
    pub is_summary: bool,
    pub model_id: String,
    pub provider_id: String,
    pub error_name: Option<String>,
    pub error_message: Option<String>,
    pub error_type: Option<String>,
    pub cwd: Option<String>,
    pub root: Option<String>,
    pub cost: Option<f64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub cached_read_tokens: Option<i64>,
    pub cached_write_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub completed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTool {
    pub message_id: Uuid,
    pub tool_name: String,
    pub enabled: bool,
}

#[derive(Debug, Error)]
pub enum MessageRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("session not found for message.session_id {0}")]
    SessionNotFound(Uuid),
    #[error("harness session id missing for session.id {0}")]
    MissingHarnessSession(Uuid),
    #[error("harness error: {0}")]
    Harness(String),
}

impl From<MessageRepoError> for Status {
    fn from(err: MessageRepoError) -> Self {
        match err {
            MessageRepoError::Database(e) => Status::internal(e.to_string()),
            MessageRepoError::SessionNotFound(id) => {
                Status::not_found(format!("session not found: {id}"))
            }
            MessageRepoError::MissingHarnessSession(id) => {
                Status::failed_precondition(format!("session missing harness session id: {id}"))
            }
            MessageRepoError::Harness(message) => Status::unavailable(message),
        }
    }
}

pub struct MessageRepo<D>
where
    D: crate::backend::db::Database,
{
    ctx: BackendContext<D>,
}

pub struct SendUserMessageResult {
    pub user_message: Message,
    pub harness_response: OpencodeMessageWithParts,
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
    ) -> Result<Vec<Message>, MessageRepoError> {
        Ok(self.ctx.db.list_messages_by_session(*session_id).await?)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Message>, MessageRepoError> {
        Ok(self.ctx.db.get_message(*id).await?)
    }

    pub async fn create(&self, message: &Message) -> Result<Message, MessageRepoError> {
        Ok(self.ctx.db.create_message(message.clone()).await?)
    }

    pub async fn update(&self, message: &Message) -> Result<Message, MessageRepoError> {
        Ok(self.ctx.db.update_message(message.clone()).await?)
    }

    pub async fn delete(&self, message_id: &Uuid) -> Result<(), MessageRepoError> {
        self.ctx.db.delete_message(*message_id).await?;
        Ok(())
    }

    pub async fn list_tools(
        &self,
        message_id: &Uuid,
    ) -> Result<Vec<MessageTool>, MessageRepoError> {
        Ok(self.ctx.db.list_message_tools(*message_id).await?)
    }

    pub async fn upsert_tool(&self, tool: &MessageTool) -> Result<MessageTool, MessageRepoError> {
        Ok(self.ctx.db.upsert_message_tool(tool.clone()).await?)
    }

    pub async fn delete_tool(
        &self,
        message_id: &Uuid,
        tool_name: &str,
    ) -> Result<(), MessageRepoError> {
        self.ctx
            .db
            .delete_message_tool(*message_id, tool_name.to_string())
            .await?;
        Ok(())
    }

    pub async fn send_user_message(
        &self,
        session_id: &Uuid,
        request: &OpencodeSendMessageRequest,
    ) -> Result<SendUserMessageResult, MessageRepoError> {
        let session = self
            .ctx
            .db
            .get_session(*session_id)
            .await?
            .ok_or(MessageRepoError::SessionNotFound(*session_id))?;

        let harness_session_id = session
            .harness_session_id
            .as_deref()
            .ok_or(MessageRepoError::MissingHarnessSession(*session_id))?;

        let now = chrono::Utc::now().naive_utc();
        let user_message_id = Uuid::new_v4();
        let user_message = Message {
            id: user_message_id,
            harness_message_id: None,
            session_id: *session_id,
            parent_message_id: None,
            role: "user".to_string(),
            title: None,
            body: Some(flatten_text_parts(&request.parts)),
            agent: request.agent.clone(),
            system_message: request.system.clone(),
            variant: None,
            is_finished_streaming: true,
            is_summary: false,
            model_id: request
                .model
                .as_ref()
                .map(|m| m.model_id.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            provider_id: request
                .model
                .as_ref()
                .map(|m| m.provider_id.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            error_name: None,
            error_message: None,
            error_type: None,
            cwd: session.dir.clone(),
            root: session.dir.clone(),
            cost: None,
            input_tokens: None,
            output_tokens: None,
            reasoning_tokens: None,
            cached_read_tokens: None,
            cached_write_tokens: None,
            total_tokens: None,
            completed_at: Some(now),
            created_at: now,
            updated_at: now,
        };

        let saved_user_message = self.ctx.db.create_message(user_message).await?;

        upsert_user_message_parts(
            &self.ctx,
            saved_user_message.session_id,
            saved_user_message.id,
            &request.parts,
            now,
        )
        .await?;

        if let Some(tools) = &request.tools {
            upsert_tools(&self.ctx, saved_user_message.id, tools).await?;
        }

        let harness_response = self
            .ctx
            .harness
            .send_message(harness_session_id, request, session.dir.as_deref())
            .await
            .map_err(|e| MessageRepoError::Harness(e.to_string()))?;

        let applier = MessageEventApplier::new(self.ctx.clone());
        applier
            .apply_message_with_parts(*session_id, harness_response.clone())
            .await
            .map_err(|e| MessageRepoError::Harness(e.to_string()))?;

        Ok(SendUserMessageResult {
            user_message: saved_user_message,
            harness_response,
        })
    }
}

fn flatten_text_parts(parts: &[OpencodePartInput]) -> String {
    let mut out = String::new();
    for part in parts {
        if let OpencodePartInput::Text { text, .. } = part {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text);
        }
    }
    out
}

async fn upsert_tools<D>(
    ctx: &BackendContext<D>,
    message_id: Uuid,
    tools: &HashMap<String, bool>,
) -> Result<(), MessageRepoError>
where
    D: Database,
{
    for (tool_name, enabled) in tools {
        ctx.db
            .upsert_message_tool(MessageTool {
                message_id,
                tool_name: tool_name.clone(),
                enabled: *enabled,
            })
            .await?;
    }
    Ok(())
}

async fn upsert_user_message_parts<D>(
    ctx: &BackendContext<D>,
    session_id: Uuid,
    message_id: Uuid,
    parts: &[OpencodePartInput],
    now: NaiveDateTime,
) -> Result<(), MessageRepoError>
where
    D: Database,
{
    for (position, part) in parts.iter().enumerate() {
        let mut row = MessagePart {
            id: Uuid::new_v4(),
            harness_part_id: None,
            session_id,
            message_id,
            position: position as i64,
            part_type: String::new(),
            text_content: None,
            synthetic: None,
            ignored: None,
            part_time_start: None,
            part_time_end: None,
            mime: None,
            filename: None,
            url: None,
            call_id: None,
            tool_name: None,
            tool_status: None,
            tool_title: None,
            tool_input_text: None,
            tool_output_text: None,
            tool_error_text: None,
            tool_time_start: None,
            tool_time_end: None,
            tool_time_compacted: None,
            step_reason: None,
            step_snapshot: None,
            step_cost: None,
            step_input_tokens: None,
            step_output_tokens: None,
            step_reasoning_tokens: None,
            step_cached_read_tokens: None,
            step_cached_write_tokens: None,
            step_total_tokens: None,
            subtask_prompt: None,
            subtask_description: None,
            subtask_agent: None,
            subtask_model_provider_id: None,
            subtask_model_id: None,
            subtask_command: None,
            retry_attempt: None,
            retry_error_message: None,
            retry_error_status_code: None,
            retry_error_is_retryable: None,
            snapshot_ref: None,
            patch_hash: None,
            compaction_auto: None,
            agent_name: None,
            agent_source_value: None,
            agent_source_start: None,
            agent_source_end: None,
            created_at: now,
            updated_at: now,
        };

        match part {
            OpencodePartInput::Text {
                text,
                synthetic,
                ignored,
                ..
            } => {
                row.part_type = "text".to_string();
                row.text_content = Some(text.clone());
                row.synthetic = *synthetic;
                row.ignored = *ignored;
            }
            OpencodePartInput::OpencodeFile {
                mime,
                filename,
                url,
                ..
            } => {
                row.part_type = "file".to_string();
                row.mime = Some(mime.clone());
                row.filename = filename.clone();
                row.url = Some(url.clone());
            }
            OpencodePartInput::Agent { name, source, .. } => {
                row.part_type = "agent".to_string();
                row.agent_name = Some(name.clone());
                if let Some(src) = source {
                    row.agent_source_value = Some(src.value.clone());
                    row.agent_source_start = Some(i64::from(src.start));
                    row.agent_source_end = Some(i64::from(src.end));
                }
            }
            OpencodePartInput::Subtask {
                prompt,
                description,
                agent,
                ..
            } => {
                row.part_type = "subtask".to_string();
                row.subtask_prompt = Some(prompt.clone());
                row.subtask_description = Some(description.clone());
                row.subtask_agent = Some(agent.clone());
            }
        }

        ctx.db.create_message_part(row).await?;
    }

    Ok(())
}
