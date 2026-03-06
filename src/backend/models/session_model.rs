use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend::{
    proto_session,
    proto_utils::{format_naive_datetime, parse_naive_datetime, parse_uuid},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_session_id: Option<Uuid>,
    pub show_in_gui: bool,
    pub name: String,
    pub harness_type: String,
    pub harness_session_id: String,
    pub dir: Option<String>,
    pub summary_additions: Option<i64>,
    pub summary_deletions: Option<i64>,
    pub summary_files: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<SessionModel> for proto_session::SessionModel {
    fn from(session: SessionModel) -> Self {
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
impl TryFrom<proto_session::SessionModel> for SessionModel {
    type Error = tonic::Status;

    fn try_from(model: proto_session::SessionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: parse_uuid("session.id", &model.id)?,
            project_id: parse_uuid("session.project_id", &model.project_id)?,
            parent_session_id: None,
            show_in_gui: model.show_in_gui,
            name: model.name,
            harness_type: "opencode".to_string(),
            harness_session_id: String::new(),
            dir: None,
            summary_additions: None,
            summary_deletions: None,
            summary_files: None,
            created_at: parse_naive_datetime("session.created_at", &model.created_at)?,
            updated_at: parse_naive_datetime("session.updated_at", &model.updated_at)?,
        })
    }
}
