use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend::{
    proto_message,
    proto_utils::{naive_datetime_to_timestamp, optional_naive_datetime_to_timestamp},
};

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
    pub harness_message_id: Option<String>,
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

impl AssistantMessage {
    pub fn new_from_harness(
        session_id: Uuid,
        user_message_id: Uuid,
        harness_message_id: &str,
    ) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            harness_message_id: Some(harness_message_id.to_string()),
            session_id,
            user_message_id,
            agent: "build".to_string(),
            model_provider_id: "unknown".to_string(),
            model_id: "unknown".to_string(),
            cwd: String::new(),
            root: String::new(),
            cost: 0.0,
            token_total: Some(0),
            token_input: 0,
            token_output: 0,
            token_reasoning: 0,
            token_cache_read: 0,
            token_cache_write: 0,
            error_message: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn ensure_harness_message_id(&mut self, harness_message_id: &str) {
        if self.harness_message_id.is_none() {
            self.harness_message_id = Some(harness_message_id.to_string());
        }
    }

    pub fn apply_harness_update(
        &mut self,
        completed_at: Option<NaiveDateTime>,
        error_message: Option<String>,
    ) {
        if completed_at.is_some() {
            self.completed_at = completed_at;
        }
        if error_message.is_some() {
            self.error_message = error_message;
        }
        self.updated_at = Utc::now().naive_utc();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessagePart {
    pub id: Uuid,
    pub harness_part_id: Option<String>,
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

fn merge_option<T>(dst: &mut Option<T>, src: Option<T>) {
    if let Some(value) = src {
        *dst = Some(value);
    }
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn json_serialized(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| serde_json::to_string(v).ok())
}

impl AssistantMessagePart {
    pub fn new_from_harness(
        session_id: Uuid,
        assistant_message_id: Uuid,
        harness_part_id: &str,
        default_part_type: &str,
    ) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            harness_part_id: Some(harness_part_id.to_string()),
            assistant_message_id,
            session_id,
            position: 0,
            part_type: default_part_type.to_string(),
            text: None,
            file_mime: None,
            file_filename: None,
            file_url: None,
            file_source_type: None,
            file_source_path: None,
            file_source_name: None,
            file_source_kind: None,
            file_source_uri: None,
            file_source_text_value: None,
            file_source_text_start: None,
            file_source_text_end: None,
            agent_name: None,
            subtask_prompt: None,
            subtask_description: None,
            subtask_agent: None,
            subtask_model_provider_id: None,
            subtask_model_id: None,
            subtask_command: None,
            tool_call_id: None,
            tool_name: None,
            tool_status: None,
            tool_input_json: None,
            tool_output_text: None,
            tool_error_text: None,
            tool_title: None,
            tool_metadata_json: None,
            tool_compacted_at: None,
            tool_state_raw: None,
            tool_state_time_start: None,
            tool_state_time_end: None,
            tool_state_time_compacted: None,
            tool_attachments_json: None,
            finish_reason: None,
            cost: None,
            token_total: None,
            token_input: None,
            token_output: None,
            token_reasoning: None,
            token_cache_read: None,
            token_cache_write: None,
            snapshot_hash: None,
            patch_hash: None,
            patch_files_json: None,
            retry_attempt: None,
            retry_error_json: None,
            retry_created_at: None,
            compaction_auto: None,
            delta_field: None,
            delta_text: None,
            part_time_start: None,
            part_time_end: None,
            part_metadata_json: None,
            text_synthetic: None,
            text_ignored: None,
            step_snapshot_hash: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn ensure_harness_part_id(&mut self, harness_part_id: &str) {
        if self.harness_part_id.is_none() {
            self.harness_part_id = Some(harness_part_id.to_string());
        }
    }

    pub fn apply_payload_json(&mut self, payload: serde_json::Value, default_part_type: &str) {
        self.part_type = payload
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default_part_type)
            .to_string();
        merge_option(&mut self.text, json_string(&payload, "text"));
        merge_option(
            &mut self.text_synthetic,
            payload
                .get("synthetic")
                .and_then(serde_json::Value::as_bool),
        );
        merge_option(
            &mut self.text_ignored,
            payload.get("ignored").and_then(serde_json::Value::as_bool),
        );

        if let Some(state) = payload.get("state") {
            merge_option(&mut self.tool_status, json_string(state, "status"));
            merge_option(&mut self.tool_input_json, json_serialized(state, "input"));
            merge_option(&mut self.tool_output_text, json_string(state, "output"));
            merge_option(&mut self.tool_error_text, json_string(state, "error"));
            merge_option(&mut self.tool_title, json_string(state, "title"));
            merge_option(&mut self.tool_state_raw, json_string(state, "raw"));
        }

        merge_option(&mut self.tool_call_id, json_string(&payload, "callID"));
        merge_option(&mut self.tool_name, json_string(&payload, "tool"));
        merge_option(&mut self.finish_reason, json_string(&payload, "reason"));
        merge_option(
            &mut self.step_snapshot_hash,
            json_string(&payload, "snapshot"),
        );
        self.updated_at = Utc::now().naive_utc();
    }

    pub fn apply_delta(&mut self, field: String, value: String) {
        self.delta_field = Some(field.clone());
        self.delta_text = Some(value.clone());
        if field == "text" {
            let current = self.text.clone().unwrap_or_default();
            self.text = Some(format!("{current}{value}"));
        }
        self.updated_at = Utc::now().naive_utc();
    }

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

impl From<AssistantMessagePart> for proto_message::AssistantMessagePartModel {
    fn from(value: AssistantMessagePart) -> Self {
        Self {
            id: value.id.to_string(),
            assistant_message_id: value.assistant_message_id.to_string(),
            session_id: value.session_id.to_string(),
            position: value.position,
            part_type: value.part_type,
            text: value.text,
            file_mime: value.file_mime,
            file_filename: value.file_filename,
            file_url: value.file_url,
            file_source_type: value.file_source_type,
            file_source_path: value.file_source_path,
            file_source_name: value.file_source_name,
            file_source_kind: value.file_source_kind,
            file_source_uri: value.file_source_uri,
            file_source_text_value: value.file_source_text_value,
            file_source_text_start: value.file_source_text_start,
            file_source_text_end: value.file_source_text_end,
            agent_name: value.agent_name,
            subtask_prompt: value.subtask_prompt,
            subtask_description: value.subtask_description,
            subtask_agent: value.subtask_agent,
            subtask_model_provider_id: value.subtask_model_provider_id,
            subtask_model_id: value.subtask_model_id,
            subtask_command: value.subtask_command,
            tool_call_id: value.tool_call_id,
            tool_name: value.tool_name,
            tool_status: value.tool_status,
            tool_input_json: value.tool_input_json,
            tool_output_text: value.tool_output_text,
            tool_error_text: value.tool_error_text,
            tool_title: value.tool_title,
            tool_metadata_json: value.tool_metadata_json,
            tool_compacted_at: value.tool_compacted_at,
            finish_reason: value.finish_reason,
            cost: value.cost,
            token_total: value.token_total,
            token_input: value.token_input,
            token_output: value.token_output,
            token_reasoning: value.token_reasoning,
            token_cache_read: value.token_cache_read,
            token_cache_write: value.token_cache_write,
            snapshot_hash: value.snapshot_hash,
            patch_hash: value.patch_hash,
            patch_files_json: value.patch_files_json,
            retry_attempt: value.retry_attempt,
            retry_error_json: value.retry_error_json,
            retry_created_at: value.retry_created_at,
            compaction_auto: value.compaction_auto,
            created_at: Some(naive_datetime_to_timestamp(value.created_at)),
            updated_at: Some(naive_datetime_to_timestamp(value.updated_at)),
            tool_state_raw: value.tool_state_raw,
            tool_state_time_start: value.tool_state_time_start,
            tool_state_time_end: value.tool_state_time_end,
            tool_state_time_compacted: value.tool_state_time_compacted,
            tool_attachments_json: value.tool_attachments_json,
            delta_field: value.delta_field,
            delta_text: value.delta_text,
            part_time_start: value.part_time_start,
            part_time_end: value.part_time_end,
            part_metadata_json: value.part_metadata_json,
            text_synthetic: value.text_synthetic,
            text_ignored: value.text_ignored,
            step_snapshot_hash: value.step_snapshot_hash,
        }
    }
}

impl From<AssistantMessage> for proto_message::AssistantMessageModel {
    fn from(value: AssistantMessage) -> Self {
        Self {
            id: value.id.to_string(),
            session_id: value.session_id.to_string(),
            user_message_id: value.user_message_id.to_string(),
            agent: value.agent,
            model_provider_id: value.model_provider_id,
            model_id: value.model_id,
            cwd: value.cwd,
            root: value.root,
            cost: value.cost,
            token_total: value.token_total,
            token_input: value.token_input,
            token_output: value.token_output,
            token_reasoning: value.token_reasoning,
            token_cache_read: value.token_cache_read,
            token_cache_write: value.token_cache_write,
            error_message: value.error_message,
            created_at: Some(naive_datetime_to_timestamp(value.created_at)),
            updated_at: Some(naive_datetime_to_timestamp(value.updated_at)),
            completed_at: optional_naive_datetime_to_timestamp(value.completed_at),
            parts: Vec::new(),
        }
    }
}
