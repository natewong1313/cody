use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    proto_message,
    proto_utils::{format_naive_datetime, parse_naive_datetime, parse_uuid},
};

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

impl From<UserMessage> for proto_message::UserMessageModel {
    fn from(value: UserMessage) -> Self {
        Self {
            id: value.id.to_string(),
            session_id: value.session_id.to_string(),
            agent: value.agent,
            model_provider_id: value.model_provider_id,
            model_id: value.model_id,
            system_prompt: value.system_prompt,
            structured_output_type: value.structured_output_type,
            tools_list: value.tools_list,
            thinking_variant: value.thinking_variant,
            created_at: format_naive_datetime(value.created_at),
            updated_at: format_naive_datetime(value.updated_at),
            parts: Vec::new(),
        }
    }
}

impl TryFrom<proto_message::UserMessageModel> for UserMessage {
    type Error = Status;

    fn try_from(value: proto_message::UserMessageModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: parse_uuid("user_message.id", &value.id)?,
            session_id: parse_uuid("user_message.session_id", &value.session_id)?,
            agent: value.agent,
            model_provider_id: value.model_provider_id,
            model_id: value.model_id,
            system_prompt: value.system_prompt,
            structured_output_type: value.structured_output_type,
            tools_list: value.tools_list,
            thinking_variant: value.thinking_variant,
            created_at: parse_naive_datetime("user_message.created_at", &value.created_at)?,
            updated_at: parse_naive_datetime("user_message.updated_at", &value.updated_at)?,
        })
    }
}
