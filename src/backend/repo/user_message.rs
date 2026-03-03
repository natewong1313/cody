use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub agent: String,
    pub model_provider_id: String,
    pub model_id: String,
    pub system_prompt: Option<String>,
    pub structured_output_type: String,
    pub tools_list: String,
    pub thinking_variant: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessagePart {
    pub id: Uuid,
    pub user_message_id: Uuid,
    pub session_id: Uuid,
    pub position: i64,
    pub part_type: String,
    pub text: Option<String>,
    pub file_name: Option<String>,
    pub file_url: Option<String>,
    pub agent_name: Option<String>,
    pub subtask_prompt: Option<String>,
    pub subtask_description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
