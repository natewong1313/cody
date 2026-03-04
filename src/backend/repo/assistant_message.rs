use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssistantPartKind {
    Text,
    Reasoning,
    Tool,
    StepStart,
    StepFinish,
    File,
    Patch,
    Snapshot,
    Subtask,
    Retry,
    Compaction,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolStatus {
    Pending,
    Running,
    Completed,
    Error,
}

impl ToolStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ToolStatus::Pending => "pending",
            ToolStatus::Running => "running",
            ToolStatus::Completed => "completed",
            ToolStatus::Error => "error",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistantPartPayload {
    Text {
        text: Option<String>,
        synthetic: Option<bool>,
        ignored: Option<bool>,
        time_start: Option<i64>,
        time_end: Option<i64>,
        metadata_json: Option<String>,
    },
    Reasoning {
        text: Option<String>,
        time_start: Option<i64>,
        time_end: Option<i64>,
        metadata_json: Option<String>,
    },
    Tool {
        call_id: Option<String>,
        name: Option<String>,
        status: Option<ToolStatus>,
        input_json: Option<String>,
        output_text: Option<String>,
        error_text: Option<String>,
        title: Option<String>,
        metadata_json: Option<String>,
        raw: Option<String>,
        time_start: Option<i64>,
        time_end: Option<i64>,
        time_compacted: Option<i64>,
        attachments_json: Option<String>,
    },
    StepStart {
        snapshot_hash: Option<String>,
    },
    StepFinish {
        finish_reason: Option<String>,
        snapshot_hash: Option<String>,
        cost: Option<f64>,
        token_total: Option<i64>,
        token_input: Option<i64>,
        token_output: Option<i64>,
        token_reasoning: Option<i64>,
        token_cache_read: Option<i64>,
        token_cache_write: Option<i64>,
    },
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_message_id: Uuid,
    pub agent: String,
    pub model_provider_id: String,
    pub model_id: String,
    pub cwd: String,
    pub root: String,
    pub cost: f64,
    pub token_total: Option<i64>,
    pub token_input: i64,
    pub token_output: i64,
    pub token_reasoning: i64,
    pub token_cache_read: i64,
    pub token_cache_write: i64,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessagePart {
    pub id: Uuid,
    pub assistant_message_id: Uuid,
    pub session_id: Uuid,
    pub position: i64,
    pub part_type: String,
    pub text: Option<String>,
    pub file_mime: Option<String>,
    pub file_filename: Option<String>,
    pub file_url: Option<String>,
    pub file_source_type: Option<String>,
    pub file_source_path: Option<String>,
    pub file_source_name: Option<String>,
    pub file_source_kind: Option<i64>,
    pub file_source_uri: Option<String>,
    pub file_source_text_value: Option<String>,
    pub file_source_text_start: Option<i64>,
    pub file_source_text_end: Option<i64>,
    pub agent_name: Option<String>,
    pub subtask_prompt: Option<String>,
    pub subtask_description: Option<String>,
    pub subtask_agent: Option<String>,
    pub subtask_model_provider_id: Option<String>,
    pub subtask_model_id: Option<String>,
    pub subtask_command: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_status: Option<String>,
    pub tool_input_json: Option<String>,
    pub tool_output_text: Option<String>,
    pub tool_error_text: Option<String>,
    pub tool_title: Option<String>,
    pub tool_metadata_json: Option<String>,
    pub tool_compacted_at: Option<i64>,
    pub tool_state_raw: Option<String>,
    pub tool_state_time_start: Option<i64>,
    pub tool_state_time_end: Option<i64>,
    pub tool_state_time_compacted: Option<i64>,
    pub tool_attachments_json: Option<String>,
    pub finish_reason: Option<String>,
    pub cost: Option<f64>,
    pub token_total: Option<i64>,
    pub token_input: Option<i64>,
    pub token_output: Option<i64>,
    pub token_reasoning: Option<i64>,
    pub token_cache_read: Option<i64>,
    pub token_cache_write: Option<i64>,
    pub snapshot_hash: Option<String>,
    pub patch_hash: Option<String>,
    pub patch_files_json: Option<String>,
    pub retry_attempt: Option<i64>,
    pub retry_error_json: Option<String>,
    pub retry_created_at: Option<i64>,
    pub compaction_auto: Option<bool>,
    pub delta_field: Option<String>,
    pub delta_text: Option<String>,
    pub part_time_start: Option<i64>,
    pub part_time_end: Option<i64>,
    pub part_metadata_json: Option<String>,
    pub text_synthetic: Option<bool>,
    pub text_ignored: Option<bool>,
    pub step_snapshot_hash: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl AssistantMessagePart {
    pub fn part_kind(&self) -> AssistantPartKind {
        match self.part_type.as_str() {
            "text" => AssistantPartKind::Text,
            "reasoning" => AssistantPartKind::Reasoning,
            "tool" => AssistantPartKind::Tool,
            "step-start" => AssistantPartKind::StepStart,
            "step-finish" => AssistantPartKind::StepFinish,
            "file" => AssistantPartKind::File,
            "patch" => AssistantPartKind::Patch,
            "snapshot" => AssistantPartKind::Snapshot,
            "subtask" => AssistantPartKind::Subtask,
            "retry" => AssistantPartKind::Retry,
            "compaction" => AssistantPartKind::Compaction,
            _ => AssistantPartKind::Unknown,
        }
    }

    pub fn tool_status_value(&self) -> Option<ToolStatus> {
        self.tool_status.as_deref().and_then(ToolStatus::from_str)
    }

    pub fn to_payload(&self) -> AssistantPartPayload {
        match self.part_kind() {
            AssistantPartKind::Text => AssistantPartPayload::Text {
                text: self.text.clone(),
                synthetic: self.text_synthetic,
                ignored: self.text_ignored,
                time_start: self.part_time_start,
                time_end: self.part_time_end,
                metadata_json: self.part_metadata_json.clone(),
            },
            AssistantPartKind::Reasoning => AssistantPartPayload::Reasoning {
                text: self.text.clone(),
                time_start: self.part_time_start,
                time_end: self.part_time_end,
                metadata_json: self.part_metadata_json.clone(),
            },
            AssistantPartKind::Tool => AssistantPartPayload::Tool {
                call_id: self.tool_call_id.clone(),
                name: self.tool_name.clone(),
                status: self.tool_status_value(),
                input_json: self.tool_input_json.clone(),
                output_text: self.tool_output_text.clone(),
                error_text: self.tool_error_text.clone(),
                title: self.tool_title.clone(),
                metadata_json: self.tool_metadata_json.clone(),
                raw: self.tool_state_raw.clone(),
                time_start: self.tool_state_time_start,
                time_end: self.tool_state_time_end,
                time_compacted: self.tool_state_time_compacted,
                attachments_json: self.tool_attachments_json.clone(),
            },
            AssistantPartKind::StepStart => AssistantPartPayload::StepStart {
                snapshot_hash: self.step_snapshot_hash.clone(),
            },
            AssistantPartKind::StepFinish => AssistantPartPayload::StepFinish {
                finish_reason: self.finish_reason.clone(),
                snapshot_hash: self.step_snapshot_hash.clone(),
                cost: self.cost,
                token_total: self.token_total,
                token_input: self.token_input,
                token_output: self.token_output,
                token_reasoning: self.token_reasoning,
                token_cache_read: self.token_cache_read,
                token_cache_write: self.token_cache_write,
            },
            _ => AssistantPartPayload::Other,
        }
    }

    pub fn apply_payload(&mut self, payload: AssistantPartPayload) {
        match payload {
            AssistantPartPayload::Text {
                text,
                synthetic,
                ignored,
                time_start,
                time_end,
                metadata_json,
            } => {
                self.part_type = "text".to_string();
                self.text = text;
                self.text_synthetic = synthetic;
                self.text_ignored = ignored;
                self.part_time_start = time_start;
                self.part_time_end = time_end;
                self.part_metadata_json = metadata_json;
            }
            AssistantPartPayload::Reasoning {
                text,
                time_start,
                time_end,
                metadata_json,
            } => {
                self.part_type = "reasoning".to_string();
                self.text = text;
                self.part_time_start = time_start;
                self.part_time_end = time_end;
                self.part_metadata_json = metadata_json;
            }
            AssistantPartPayload::Tool {
                call_id,
                name,
                status,
                input_json,
                output_text,
                error_text,
                title,
                metadata_json,
                raw,
                time_start,
                time_end,
                time_compacted,
                attachments_json,
            } => {
                self.part_type = "tool".to_string();
                self.tool_call_id = call_id;
                self.tool_name = name;
                self.tool_status = status.map(ToolStatus::as_str).map(str::to_string);
                self.tool_input_json = input_json;
                self.tool_output_text = output_text;
                self.tool_error_text = error_text;
                self.tool_title = title;
                self.tool_metadata_json = metadata_json;
                self.tool_state_raw = raw;
                self.tool_state_time_start = time_start;
                self.tool_state_time_end = time_end;
                self.tool_state_time_compacted = time_compacted;
                self.tool_attachments_json = attachments_json;
            }
            AssistantPartPayload::StepStart { snapshot_hash } => {
                self.part_type = "step-start".to_string();
                self.step_snapshot_hash = snapshot_hash;
            }
            AssistantPartPayload::StepFinish {
                finish_reason,
                snapshot_hash,
                cost,
                token_total,
                token_input,
                token_output,
                token_reasoning,
                token_cache_read,
                token_cache_write,
            } => {
                self.part_type = "step-finish".to_string();
                self.finish_reason = finish_reason;
                self.step_snapshot_hash = snapshot_hash;
                self.cost = cost;
                self.token_total = token_total;
                self.token_input = token_input;
                self.token_output = token_output;
                self.token_reasoning = token_reasoning;
                self.token_cache_read = token_cache_read;
                self.token_cache_write = token_cache_write;
            }
            AssistantPartPayload::Other => {}
        }
    }
}
