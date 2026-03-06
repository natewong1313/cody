use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tonic::Status;
use uuid::Uuid;

use crate::backend::{
    proto_message,
    proto_utils::{naive_datetime_to_timestamp, parse_uuid, timestamp_to_naive_datetime},
};

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

impl From<UserMessagePart> for proto_message::UserMessagePartModel {
    fn from(value: UserMessagePart) -> Self {
        Self {
            id: value.id.to_string(),
            user_message_id: value.user_message_id.to_string(),
            session_id: value.session_id.to_string(),
            position: value.position,
            part_type: value.part_type,
            text: value.text,
            file_name: value.file_name,
            file_url: value.file_url,
            agent_name: value.agent_name,
            subtask_prompt: value.subtask_prompt,
            subtask_description: value.subtask_description,
            created_at: Some(naive_datetime_to_timestamp(value.created_at)),
            updated_at: Some(naive_datetime_to_timestamp(value.updated_at)),
        }
    }
}

impl TryFrom<proto_message::UserMessagePartModel> for UserMessagePart {
    type Error = Status;

    fn try_from(value: proto_message::UserMessagePartModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: parse_uuid("user_message_part.id", &value.id)?,
            user_message_id: parse_uuid(
                "user_message_part.user_message_id",
                &value.user_message_id,
            )?,
            session_id: parse_uuid("user_message_part.session_id", &value.session_id)?,
            position: value.position,
            part_type: value.part_type,
            text: value.text,
            file_name: value.file_name,
            file_url: value.file_url,
            agent_name: value.agent_name,
            subtask_prompt: value.subtask_prompt,
            subtask_description: value.subtask_description,
            created_at: timestamp_to_naive_datetime(
                "user_message_part.created_at",
                value.created_at,
            )?,
            updated_at: timestamp_to_naive_datetime(
                "user_message_part.updated_at",
                value.updated_at,
            )?,
        })
    }
}
