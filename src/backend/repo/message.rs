use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: Uuid,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageTool {
    pub message_id: Uuid,
    pub tool_name: String,
    pub enabled: bool,
}
