use chrono::{NaiveDateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::backend::{
    BackendContext, Message, MessagePart,
    db::{Database, DatabaseError},
    harness::{
        OpencodeEventPayload, OpencodeGlobalEvent, OpencodeMessage, OpencodeMessageError,
        OpencodePart, OpencodeToolState,
    },
};

#[derive(Debug, Error)]
pub enum MessageEventApplyError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("session not found for harness_session_id {0}")]
    SessionNotFound(String),
    #[error("message not found for harness_message_id {0}")]
    MessageNotFound(String),
    #[error("invalid role in message row {0}")]
    InvalidRole(String),
}

#[derive(Debug, Clone)]
pub enum MessageDiffEvent {
    MessageUpserted {
        session_id: Uuid,
        message: Message,
    },
    MessagePartUpserted {
        session_id: Uuid,
        part: MessagePart,
        delta: Option<String>,
    },
    MessageRemoved {
        session_id: Uuid,
        harness_message_id: String,
    },
    SessionIdle {
        session_id: Uuid,
    },
}

pub struct MessageEventApplier<D>
where
    D: Database,
{
    ctx: BackendContext<D>,
}

impl<D> MessageEventApplier<D>
where
    D: Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn apply(
        &self,
        event: OpencodeGlobalEvent,
    ) -> Result<Option<MessageDiffEvent>, MessageEventApplyError> {
        match event.payload {
            OpencodeEventPayload::MessageUpdated { props } => {
                let local_session = self
                    .ctx
                    .db
                    .get_session_by_harness_session_id(props.info.session_id().to_string())
                    .await?
                    .ok_or_else(|| {
                        MessageEventApplyError::SessionNotFound(props.info.session_id().to_string())
                    })?;

                self.apply_message_updated(local_session.id, props.info)
                    .await
            }
            OpencodeEventPayload::MessagePartUpdated { props } => {
                let local_session = self
                    .ctx
                    .db
                    .get_session_by_harness_session_id(props.part.session_id().to_string())
                    .await?
                    .ok_or_else(|| {
                        MessageEventApplyError::SessionNotFound(props.part.session_id().to_string())
                    })?;

                self.apply_part_updated(local_session.id, props.part, props.delta)
                    .await
            }
            OpencodeEventPayload::MessageRemoved { props } => {
                let harness_message_id = props.message_id.clone();
                let local_session = self
                    .ctx
                    .db
                    .get_session_by_harness_session_id(props.session_id.clone())
                    .await?
                    .ok_or_else(|| {
                        MessageEventApplyError::SessionNotFound(props.session_id.clone())
                    })?;
                self.ctx
                    .db
                    .delete_message_by_harness_message_id(local_session.id, harness_message_id.clone())
                    .await?;
                Ok(Some(MessageDiffEvent::MessageRemoved {
                    session_id: local_session.id,
                    harness_message_id,
                }))
            }
            OpencodeEventPayload::SessionIdle { props } => {
                let local_session = self
                    .ctx
                    .db
                    .get_session_by_harness_session_id(props.session_id.clone())
                    .await?
                    .ok_or_else(|| {
                        MessageEventApplyError::SessionNotFound(props.session_id.clone())
                    })?;

                self.ctx
                    .db
                    .mark_session_assistant_messages_finished(
                        local_session.id,
                        Utc::now().naive_utc(),
                    )
                    .await?;
                Ok(Some(MessageDiffEvent::SessionIdle {
                    session_id: local_session.id,
                }))
            }
        }
    }

    async fn apply_message_updated(
        &self,
        local_session_id: Uuid,
        info: OpencodeMessage,
    ) -> Result<Option<MessageDiffEvent>, MessageEventApplyError> {
        let harness_message_id = info.id().to_string();
        let existing = self
            .ctx
            .db
            .get_message_by_harness_message_id(local_session_id, harness_message_id.clone())
            .await?;
        let had_existing = existing.is_some();

        let now = Utc::now().naive_utc();
        let mut row = existing.clone().unwrap_or_else(|| Message {
            id: Uuid::new_v4(),
            harness_message_id: Some(harness_message_id.clone()),
            session_id: local_session_id,
            parent_message_id: None,
            role: "assistant".to_string(),
            title: None,
            body: None,
            agent: None,
            system_message: None,
            variant: None,
            is_finished_streaming: false,
            is_summary: false,
            model_id: "unknown".to_string(),
            provider_id: "unknown".to_string(),
            error_name: None,
            error_message: None,
            error_type: None,
            cwd: None,
            root: None,
            cost: None,
            input_tokens: None,
            output_tokens: None,
            reasoning_tokens: None,
            cached_read_tokens: None,
            cached_write_tokens: None,
            total_tokens: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        });

        row.harness_message_id = Some(harness_message_id);
        row.session_id = local_session_id;

        match info {
            OpencodeMessage::User(user) => {
                row.role = "user".to_string();
                row.agent = Some(user.agent);
                row.system_message = user.system;
                row.model_id = user.model.model_id;
                row.provider_id = user.model.provider_id;
                row.created_at = from_ms(user.time.created);
                row.completed_at = Some(row.created_at);
                row.is_finished_streaming = true;
            }
            OpencodeMessage::Assistant(assistant) => {
                row.role = "assistant".to_string();
                row.model_id = assistant.model_id;
                row.provider_id = assistant.provider_id;
                row.parent_message_id = self
                    .ctx
                    .db
                    .get_message_by_harness_message_id(
                        local_session_id,
                        assistant.parent_id.clone(),
                    )
                    .await?
                    .map(|m| m.id);
                row.cwd = Some(assistant.path.cwd);
                row.root = Some(assistant.path.root);
                row.cost = Some(assistant.cost);
                row.input_tokens = Some(i64::from(assistant.tokens.input));
                row.output_tokens = Some(i64::from(assistant.tokens.output));
                row.reasoning_tokens = Some(i64::from(assistant.tokens.reasoning));
                row.cached_read_tokens = Some(i64::from(assistant.tokens.cache.read));
                row.cached_write_tokens = Some(i64::from(assistant.tokens.cache.write));
                row.total_tokens = Some(
                    i64::from(assistant.tokens.input)
                        + i64::from(assistant.tokens.output)
                        + i64::from(assistant.tokens.reasoning),
                );
                row.created_at = from_ms(assistant.time.created);
                row.completed_at = assistant.time.completed.map(from_ms);
                row.is_finished_streaming = assistant.time.completed.is_some();

                if let Some(err) = assistant.error {
                    apply_assistant_error(&mut row, err);
                } else {
                    row.error_name = None;
                    row.error_message = None;
                    row.error_type = None;
                }
            }
        }

        row.updated_at = now;
        let saved = if had_existing {
            self.ctx.db.update_message(row).await?
        } else {
            self.ctx.db.create_message(row).await?
        };
        Ok(Some(MessageDiffEvent::MessageUpserted {
            session_id: local_session_id,
            message: saved,
        }))
    }

    async fn apply_part_updated(
        &self,
        local_session_id: Uuid,
        part: OpencodePart,
        delta: Option<String>,
    ) -> Result<Option<MessageDiffEvent>, MessageEventApplyError> {
        let harness_message_id = part.message_id().to_string();
        let message = self
            .ctx
            .db
            .get_message_by_harness_message_id(local_session_id, harness_message_id.clone())
            .await?
            .ok_or(MessageEventApplyError::MessageNotFound(harness_message_id))?;

        let harness_part_id = part_id(&part).to_string();
        let existing = self
            .ctx
            .db
            .get_message_part_by_harness_part_id(message.id, harness_part_id.clone())
            .await?;
        let had_existing = existing.is_some();

        if let Some(existing_part) = existing.as_ref()
            && let Some(text_delta) = delta.clone()
            && is_text_like_part(&part)
        {
            let updated = self.ctx
                .db
                .append_message_part_text_delta(existing_part.id, text_delta)
                .await?;
            return Ok(Some(MessageDiffEvent::MessagePartUpserted {
                session_id: local_session_id,
                part: updated,
                delta,
            }));
        }

        let now = Utc::now().naive_utc();
        let mut row = existing.unwrap_or_else(|| empty_part(message.id, local_session_id, now));
        row.harness_part_id = Some(harness_part_id);
        row.updated_at = now;

        match part {
            OpencodePart::Text(p) => {
                row.part_type = "text".to_string();
                row.text_content = Some(p.text);
                row.synthetic = p.synthetic;
                row.ignored = p.ignored;
            }
            OpencodePart::Reasoning(p) => {
                row.part_type = "reasoning".to_string();
                row.text_content = Some(p.text);
            }
            OpencodePart::Tool(p) => {
                row.part_type = "tool".to_string();
                row.call_id = Some(p.call_id);
                row.tool_name = Some(p.tool);
                match p.state {
                    OpencodeToolState::Pending(s) => {
                        row.tool_status = Some(s.status);
                        row.tool_input_text = Some(s.input.to_string());
                    }
                    OpencodeToolState::Running(s) => {
                        row.tool_status = Some(s.status);
                        row.tool_input_text = Some(s.input.to_string());
                        row.tool_title = s.title;
                    }
                    OpencodeToolState::Completed(s) => {
                        row.tool_status = Some(s.status);
                        row.tool_input_text = Some(s.input.to_string());
                        row.tool_output_text = Some(s.output);
                        row.tool_title = Some(s.title);
                    }
                    OpencodeToolState::Error(s) => {
                        row.tool_status = Some(s.status);
                        row.tool_input_text = Some(s.input.to_string());
                        row.tool_error_text = Some(s.error);
                    }
                }
            }
        }

        let saved = if had_existing {
            self.ctx.db.update_message_part(row).await?
        } else {
            self.ctx.db.create_message_part(row).await?
        };
        Ok(Some(MessageDiffEvent::MessagePartUpserted {
            session_id: local_session_id,
            part: saved,
            delta,
        }))
    }
}

fn from_ms(ms: i64) -> NaiveDateTime {
    if let Some(dt) = chrono::DateTime::from_timestamp_millis(ms) {
        dt.naive_utc()
    } else {
        Utc::now().naive_utc()
    }
}

fn apply_assistant_error(row: &mut Message, err: OpencodeMessageError) {
    match err {
        OpencodeMessageError::ProviderAuth(e) => {
            row.error_name = Some(e.name);
            row.error_message = Some(e.data.message);
            row.error_type = Some("provider_auth".to_string());
        }
        OpencodeMessageError::Unknown(e) => {
            row.error_name = Some(e.name);
            row.error_message = Some(e.data.message);
            row.error_type = Some("unknown".to_string());
        }
        OpencodeMessageError::Api(e) => {
            row.error_name = Some(e.name);
            row.error_message = Some(e.data.message);
            row.error_type = Some("api".to_string());
        }
    }
}

fn empty_part(message_id: Uuid, session_id: Uuid, now: NaiveDateTime) -> MessagePart {
    MessagePart {
        id: Uuid::new_v4(),
        harness_part_id: None,
        session_id,
        message_id,
        position: 0,
        part_type: "text".to_string(),
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
    }
}

fn is_text_like_part(part: &OpencodePart) -> bool {
    matches!(part, OpencodePart::Text(_) | OpencodePart::Reasoning(_))
}

fn part_id(part: &OpencodePart) -> &str {
    match part {
        OpencodePart::Text(p) => &p.id,
        OpencodePart::Reasoning(p) => &p.id,
        OpencodePart::Tool(p) => &p.id,
    }
}
