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
