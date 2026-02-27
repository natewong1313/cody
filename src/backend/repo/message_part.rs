use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessagePart {
    pub id: Uuid,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessagePartAttachment {
    pub id: Uuid,
    pub part_id: Uuid,
    pub mime: String,
    pub url: String,
    pub filename: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessagePartPatchFile {
    pub part_id: Uuid,
    pub file_path: String,
}
