use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, DatabaseError},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    pub id: Uuid,
    pub harness_part_id: Option<String>,
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub position: i64,
    pub part_type: String,
    pub text_content: Option<String>,
    pub synthetic: Option<bool>,
    pub ignored: Option<bool>,
    pub part_time_start: Option<String>,
    pub part_time_end: Option<String>,
    pub mime: Option<String>,
    pub filename: Option<String>,
    pub url: Option<String>,
    pub call_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_status: Option<String>,
    pub tool_title: Option<String>,
    pub tool_input_text: Option<String>,
    pub tool_output_text: Option<String>,
    pub tool_error_text: Option<String>,
    pub tool_time_start: Option<String>,
    pub tool_time_end: Option<String>,
    pub tool_time_compacted: Option<String>,
    pub step_reason: Option<String>,
    pub step_snapshot: Option<String>,
    pub step_cost: Option<f64>,
    pub step_input_tokens: Option<i64>,
    pub step_output_tokens: Option<i64>,
    pub step_reasoning_tokens: Option<i64>,
    pub step_cached_read_tokens: Option<i64>,
    pub step_cached_write_tokens: Option<i64>,
    pub step_total_tokens: Option<i64>,
    pub subtask_prompt: Option<String>,
    pub subtask_description: Option<String>,
    pub subtask_agent: Option<String>,
    pub subtask_model_provider_id: Option<String>,
    pub subtask_model_id: Option<String>,
    pub subtask_command: Option<String>,
    pub retry_attempt: Option<i64>,
    pub retry_error_message: Option<String>,
    pub retry_error_status_code: Option<i64>,
    pub retry_error_is_retryable: Option<bool>,
    pub snapshot_ref: Option<String>,
    pub patch_hash: Option<String>,
    pub compaction_auto: Option<bool>,
    pub agent_name: Option<String>,
    pub agent_source_value: Option<String>,
    pub agent_source_start: Option<i64>,
    pub agent_source_end: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePartAttachment {
    pub id: Uuid,
    pub part_id: Uuid,
    pub mime: String,
    pub url: String,
    pub filename: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePartFileSource {
    pub part_id: Uuid,
    pub source_type: String,
    pub path: Option<String>,
    pub symbol_name: Option<String>,
    pub symbol_kind: Option<i64>,
    pub range_start_line: Option<i64>,
    pub range_start_col: Option<i64>,
    pub range_end_line: Option<i64>,
    pub range_end_col: Option<i64>,
    pub client_name: Option<String>,
    pub uri: Option<String>,
    pub source_text_value: Option<String>,
    pub source_text_start: Option<i64>,
    pub source_text_end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePartPatchFile {
    pub part_id: Uuid,
    pub file_path: String,
}

#[derive(Debug, Error)]
pub enum MessagePartRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
}

impl From<MessagePartRepoError> for Status {
    fn from(err: MessagePartRepoError) -> Self {
        match err {
            MessagePartRepoError::Database(e) => Status::internal(e.to_string()),
        }
    }
}

pub struct MessagePartRepo<D>
where
    D: Database,
{
    ctx: BackendContext<D>,
}

impl<D> MessagePartRepo<D>
where
    D: Database,
{
    pub fn new(ctx: BackendContext<D>) -> Self {
        Self { ctx }
    }

    pub async fn list_by_message(
        &self,
        message_id: &Uuid,
    ) -> Result<Vec<MessagePart>, MessagePartRepoError> {
        Ok(self
            .ctx
            .db
            .list_message_parts_by_message(*message_id)
            .await?)
    }

    pub async fn get(&self, part_id: &Uuid) -> Result<Option<MessagePart>, MessagePartRepoError> {
        Ok(self.ctx.db.get_message_part(*part_id).await?)
    }

    pub async fn create(&self, part: &MessagePart) -> Result<MessagePart, MessagePartRepoError> {
        Ok(self.ctx.db.create_message_part(part.clone()).await?)
    }

    pub async fn update(&self, part: &MessagePart) -> Result<MessagePart, MessagePartRepoError> {
        Ok(self.ctx.db.update_message_part(part.clone()).await?)
    }

    pub async fn delete(&self, part_id: &Uuid) -> Result<(), MessagePartRepoError> {
        self.ctx.db.delete_message_part(*part_id).await?;
        Ok(())
    }

    pub async fn list_attachments(
        &self,
        part_id: &Uuid,
    ) -> Result<Vec<MessagePartAttachment>, MessagePartRepoError> {
        Ok(self.ctx.db.list_message_part_attachments(*part_id).await?)
    }

    pub async fn create_attachment(
        &self,
        attachment: &MessagePartAttachment,
    ) -> Result<MessagePartAttachment, MessagePartRepoError> {
        Ok(self
            .ctx
            .db
            .create_message_part_attachment(attachment.clone())
            .await?)
    }

    pub async fn delete_attachment(
        &self,
        attachment_id: &Uuid,
    ) -> Result<(), MessagePartRepoError> {
        self.ctx
            .db
            .delete_message_part_attachment(*attachment_id)
            .await?;
        Ok(())
    }

    pub async fn get_file_source(
        &self,
        part_id: &Uuid,
    ) -> Result<Option<MessagePartFileSource>, MessagePartRepoError> {
        Ok(self.ctx.db.get_message_part_file_source(*part_id).await?)
    }

    pub async fn upsert_file_source(
        &self,
        source: &MessagePartFileSource,
    ) -> Result<MessagePartFileSource, MessagePartRepoError> {
        Ok(self
            .ctx
            .db
            .upsert_message_part_file_source(source.clone())
            .await?)
    }

    pub async fn delete_file_source(&self, part_id: &Uuid) -> Result<(), MessagePartRepoError> {
        self.ctx
            .db
            .delete_message_part_file_source(*part_id)
            .await?;
        Ok(())
    }

    pub async fn list_patch_files(
        &self,
        part_id: &Uuid,
    ) -> Result<Vec<MessagePartPatchFile>, MessagePartRepoError> {
        Ok(self.ctx.db.list_message_part_patch_files(*part_id).await?)
    }

    pub async fn create_patch_file(
        &self,
        patch_file: &MessagePartPatchFile,
    ) -> Result<MessagePartPatchFile, MessagePartRepoError> {
        Ok(self
            .ctx
            .db
            .create_message_part_patch_file(patch_file.clone())
            .await?)
    }

    pub async fn delete_patch_file(
        &self,
        part_id: &Uuid,
        file_path: &str,
    ) -> Result<(), MessagePartRepoError> {
        self.ctx
            .db
            .delete_message_part_patch_file(*part_id, file_path.to_string())
            .await?;
        Ok(())
    }
}
