use serde_rusqlite::{from_rows, to_params_named, to_params_named_with_fields};
use tokio_rusqlite::named_params;
use tokio_rusqlite::rusqlite::Connection;
use uuid::Uuid;

use crate::backend::{db::DatabaseError, repo::assistant_message::AssistantMessagePart};

const ASSISTANT_MESSAGE_PART_COLUMNS: &str = "
id, harness_part_id, assistant_message_id, session_id, position, part_type, text,
file_mime, file_filename, file_url,
file_source_type, file_source_path, file_source_name, file_source_kind, file_source_uri,
file_source_text_value, file_source_text_start, file_source_text_end,
agent_name, subtask_prompt, subtask_description, subtask_agent,
subtask_model_provider_id, subtask_model_id, subtask_command,
tool_call_id, tool_name, tool_status, tool_input_json, tool_output_text, tool_error_text,
tool_title, tool_metadata_json, tool_compacted_at,
tool_state_raw, tool_state_time_start, tool_state_time_end, tool_state_time_compacted,
tool_attachments_json,
finish_reason, cost, token_total, token_input, token_output, token_reasoning,
token_cache_read, token_cache_write,
snapshot_hash, patch_hash, patch_files_json, retry_attempt, retry_error_json, retry_created_at,
compaction_auto,
delta_field, delta_text, part_time_start, part_time_end, part_metadata_json,
text_synthetic, text_ignored, step_snapshot_hash,
created_at, updated_at
";

pub fn get(
    conn: &Connection,
    part_id: Uuid,
) -> Result<Option<AssistantMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT * FROM assistant_message_part WHERE id = :id")?;
    let mut rows =
        from_rows::<AssistantMessagePart>(stmt.query(named_params! {":id": part_id.to_string()})?);
    Ok(rows.next().transpose()?)
}

pub fn get_by_harness_id(
    conn: &Connection,
    assistant_message_id: Uuid,
    harness_part_id: &str,
) -> Result<Option<AssistantMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM assistant_message_part WHERE assistant_message_id = :assistant_message_id AND harness_part_id = :harness_part_id",
    )?;
    let mut rows = from_rows::<AssistantMessagePart>(stmt.query(named_params! {
        ":assistant_message_id": assistant_message_id.to_string(),
        ":harness_part_id": harness_part_id,
    })?);
    Ok(rows.next().transpose()?)
}

pub fn create(
    conn: &Connection,
    part: &AssistantMessagePart,
) -> Result<AssistantMessagePart, DatabaseError> {
    let params = to_params_named(part)?;
    let mut stmt = conn.prepare(&format!(
        "INSERT INTO assistant_message_part ({ASSISTANT_MESSAGE_PART_COLUMNS})
         VALUES (
            :id, :harness_part_id, :assistant_message_id, :session_id, :position, :part_type, :text,
            :file_mime, :file_filename, :file_url,
            :file_source_type, :file_source_path, :file_source_name, :file_source_kind, :file_source_uri,
            :file_source_text_value, :file_source_text_start, :file_source_text_end,
            :agent_name, :subtask_prompt, :subtask_description, :subtask_agent,
            :subtask_model_provider_id, :subtask_model_id, :subtask_command,
            :tool_call_id, :tool_name, :tool_status, :tool_input_json, :tool_output_text, :tool_error_text,
            :tool_title, :tool_metadata_json, :tool_compacted_at,
            :tool_state_raw, :tool_state_time_start, :tool_state_time_end, :tool_state_time_compacted,
            :tool_attachments_json,
            :finish_reason, :cost, :token_total, :token_input, :token_output, :token_reasoning,
            :token_cache_read, :token_cache_write,
            :snapshot_hash, :patch_hash, :patch_files_json, :retry_attempt, :retry_error_json, :retry_created_at,
            :compaction_auto,
            :delta_field, :delta_text, :part_time_start, :part_time_end, :part_metadata_json,
            :text_synthetic, :text_ignored, :step_snapshot_hash,
            :created_at, :updated_at
         )
         RETURNING {ASSISTANT_MESSAGE_PART_COLUMNS}"
    ))?;
    let rows = from_rows::<AssistantMessagePart>(stmt.query(params.to_slice().as_slice())?);
    super::expect_one_returned_row("create_assistant_message_part", rows)
}

pub fn update(
    conn: &Connection,
    part: &AssistantMessagePart,
) -> Result<AssistantMessagePart, DatabaseError> {
    let mut updated = part.clone();
    updated.updated_at = chrono::Utc::now().naive_utc();

    let params = to_params_named_with_fields(
        &updated,
        &[
            "id",
            "harness_part_id",
            "assistant_message_id",
            "session_id",
            "position",
            "part_type",
            "text",
            "file_mime",
            "file_filename",
            "file_url",
            "file_source_type",
            "file_source_path",
            "file_source_name",
            "file_source_kind",
            "file_source_uri",
            "file_source_text_value",
            "file_source_text_start",
            "file_source_text_end",
            "agent_name",
            "subtask_prompt",
            "subtask_description",
            "subtask_agent",
            "subtask_model_provider_id",
            "subtask_model_id",
            "subtask_command",
            "tool_call_id",
            "tool_name",
            "tool_status",
            "tool_input_json",
            "tool_output_text",
            "tool_error_text",
            "tool_title",
            "tool_metadata_json",
            "tool_compacted_at",
            "tool_state_raw",
            "tool_state_time_start",
            "tool_state_time_end",
            "tool_state_time_compacted",
            "tool_attachments_json",
            "finish_reason",
            "cost",
            "token_total",
            "token_input",
            "token_output",
            "token_reasoning",
            "token_cache_read",
            "token_cache_write",
            "snapshot_hash",
            "patch_hash",
            "patch_files_json",
            "retry_attempt",
            "retry_error_json",
            "retry_created_at",
            "compaction_auto",
            "delta_field",
            "delta_text",
            "part_time_start",
            "part_time_end",
            "part_metadata_json",
            "text_synthetic",
            "text_ignored",
            "step_snapshot_hash",
            "updated_at",
        ],
    )?;

    let mut stmt = conn.prepare(
        "UPDATE assistant_message_part
         SET
            harness_part_id = :harness_part_id,
            assistant_message_id = :assistant_message_id,
            session_id = :session_id,
            position = :position,
            part_type = :part_type,
            text = :text,
            file_mime = :file_mime,
            file_filename = :file_filename,
            file_url = :file_url,
            file_source_type = :file_source_type,
            file_source_path = :file_source_path,
            file_source_name = :file_source_name,
            file_source_kind = :file_source_kind,
            file_source_uri = :file_source_uri,
            file_source_text_value = :file_source_text_value,
            file_source_text_start = :file_source_text_start,
            file_source_text_end = :file_source_text_end,
            agent_name = :agent_name,
            subtask_prompt = :subtask_prompt,
            subtask_description = :subtask_description,
            subtask_agent = :subtask_agent,
            subtask_model_provider_id = :subtask_model_provider_id,
            subtask_model_id = :subtask_model_id,
            subtask_command = :subtask_command,
            tool_call_id = :tool_call_id,
            tool_name = :tool_name,
            tool_status = :tool_status,
            tool_input_json = :tool_input_json,
            tool_output_text = :tool_output_text,
            tool_error_text = :tool_error_text,
            tool_title = :tool_title,
            tool_metadata_json = :tool_metadata_json,
            tool_compacted_at = :tool_compacted_at,
            tool_state_raw = :tool_state_raw,
            tool_state_time_start = :tool_state_time_start,
            tool_state_time_end = :tool_state_time_end,
            tool_state_time_compacted = :tool_state_time_compacted,
            tool_attachments_json = :tool_attachments_json,
            finish_reason = :finish_reason,
            cost = :cost,
            token_total = :token_total,
            token_input = :token_input,
            token_output = :token_output,
            token_reasoning = :token_reasoning,
            token_cache_read = :token_cache_read,
            token_cache_write = :token_cache_write,
            snapshot_hash = :snapshot_hash,
            patch_hash = :patch_hash,
            patch_files_json = :patch_files_json,
            retry_attempt = :retry_attempt,
            retry_error_json = :retry_error_json,
            retry_created_at = :retry_created_at,
            compaction_auto = :compaction_auto,
            delta_field = :delta_field,
            delta_text = :delta_text,
            part_time_start = :part_time_start,
            part_time_end = :part_time_end,
            part_metadata_json = :part_metadata_json,
            text_synthetic = :text_synthetic,
            text_ignored = :text_ignored,
            step_snapshot_hash = :step_snapshot_hash,
            updated_at = :updated_at
         WHERE id = :id
         RETURNING *",
    )?;
    let rows = from_rows::<AssistantMessagePart>(stmt.query(params.to_slice().as_slice())?);
    super::expect_one_returned_row("update_assistant_message_part", rows)
}

pub fn delete(conn: &Connection, part_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM assistant_message_part WHERE id = :id",
        named_params! {":id": part_id.to_string()},
    )?;
    super::assert_one_row_affected("delete_assistant_message_part", rows)
}
