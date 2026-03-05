use tokio_rusqlite::rusqlite::{self, Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::backend::{
    db::{
        DatabaseError,
        sqlite::{assert_one_row_affected, check_returning_row_error, now_utc_string},
    },
    repo::assistant_message::AssistantMessagePart,
};

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

pub fn row_to_assistant_message_part(row: &Row) -> Result<AssistantMessagePart, rusqlite::Error> {
    Ok(AssistantMessagePart {
        id: row.get(0)?,
        harness_part_id: row.get(1)?,
        assistant_message_id: row.get(2)?,
        session_id: row.get(3)?,
        position: row.get(4)?,
        part_type: row.get(5)?,
        text: row.get(6)?,
        file_mime: row.get(7)?,
        file_filename: row.get(8)?,
        file_url: row.get(9)?,
        file_source_type: row.get(10)?,
        file_source_path: row.get(11)?,
        file_source_name: row.get(12)?,
        file_source_kind: row.get(13)?,
        file_source_uri: row.get(14)?,
        file_source_text_value: row.get(15)?,
        file_source_text_start: row.get(16)?,
        file_source_text_end: row.get(17)?,
        agent_name: row.get(18)?,
        subtask_prompt: row.get(19)?,
        subtask_description: row.get(20)?,
        subtask_agent: row.get(21)?,
        subtask_model_provider_id: row.get(22)?,
        subtask_model_id: row.get(23)?,
        subtask_command: row.get(24)?,
        tool_call_id: row.get(25)?,
        tool_name: row.get(26)?,
        tool_status: row.get(27)?,
        tool_input_json: row.get(28)?,
        tool_output_text: row.get(29)?,
        tool_error_text: row.get(30)?,
        tool_title: row.get(31)?,
        tool_metadata_json: row.get(32)?,
        tool_compacted_at: row.get(33)?,
        tool_state_raw: row.get(34)?,
        tool_state_time_start: row.get(35)?,
        tool_state_time_end: row.get(36)?,
        tool_state_time_compacted: row.get(37)?,
        tool_attachments_json: row.get(38)?,
        finish_reason: row.get(39)?,
        cost: row.get(40)?,
        token_total: row.get(41)?,
        token_input: row.get(42)?,
        token_output: row.get(43)?,
        token_reasoning: row.get(44)?,
        token_cache_read: row.get(45)?,
        token_cache_write: row.get(46)?,
        snapshot_hash: row.get(47)?,
        patch_hash: row.get(48)?,
        patch_files_json: row.get(49)?,
        retry_attempt: row.get(50)?,
        retry_error_json: row.get(51)?,
        retry_created_at: row.get(52)?,
        compaction_auto: row.get(53)?,
        delta_field: row.get(54)?,
        delta_text: row.get(55)?,
        part_time_start: row.get(56)?,
        part_time_end: row.get(57)?,
        part_metadata_json: row.get(58)?,
        text_synthetic: row.get(59)?,
        text_ignored: row.get(60)?,
        step_snapshot_hash: row.get(61)?,
        created_at: row.get(62)?,
        updated_at: row.get(63)?,
    })
}

pub fn get(
    conn: &Connection,
    part_id: Uuid,
) -> Result<Option<AssistantMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ASSISTANT_MESSAGE_PART_COLUMNS}
         FROM assistant_message_part
         WHERE id = ?1"
    ))?;
    Ok(stmt
        .query_row([part_id], row_to_assistant_message_part)
        .optional()?)
}

/// Select a message part using its parent message id and associated harness part id
pub fn get_by_harness_id(
    conn: &Connection,
    assistant_message_id: Uuid,
    harness_part_id: &str,
) -> Result<Option<AssistantMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ASSISTANT_MESSAGE_PART_COLUMNS}
         FROM assistant_message_part
         WHERE assistant_message_id = ?1 AND harness_part_id = ?2"
    ))?;
    Ok(stmt
        .query_row(
            params![assistant_message_id, harness_part_id],
            row_to_assistant_message_part,
        )
        .optional()?)
}

pub fn create(
    conn: &Connection,
    part: &AssistantMessagePart,
) -> Result<AssistantMessagePart, DatabaseError> {
    let created = conn.query_row(
        &format!(
            "INSERT INTO assistant_message_part ({ASSISTANT_MESSAGE_PART_COLUMNS})
              VALUES (
                 ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18,
                ?19, ?20, ?21, ?22, ?23, ?24, ?25,
                ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34,
                ?35, ?36, ?37, ?38, ?39,
                ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47,
                ?48, ?49, ?50, ?51, ?52, ?53,
                 ?54, ?55, ?56, ?57, ?58,
                 ?59, ?60, ?61,
                 ?62, ?63, ?64
              )
              RETURNING {ASSISTANT_MESSAGE_PART_COLUMNS}"
        ),
        params![
            &part.id,
            &part.harness_part_id,
            &part.assistant_message_id,
            &part.session_id,
            &part.position,
            &part.part_type,
            &part.text,
            &part.file_mime,
            &part.file_filename,
            &part.file_url,
            &part.file_source_type,
            &part.file_source_path,
            &part.file_source_name,
            &part.file_source_kind,
            &part.file_source_uri,
            &part.file_source_text_value,
            &part.file_source_text_start,
            &part.file_source_text_end,
            &part.agent_name,
            &part.subtask_prompt,
            &part.subtask_description,
            &part.subtask_agent,
            &part.subtask_model_provider_id,
            &part.subtask_model_id,
            &part.subtask_command,
            &part.tool_call_id,
            &part.tool_name,
            &part.tool_status,
            &part.tool_input_json,
            &part.tool_output_text,
            &part.tool_error_text,
            &part.tool_title,
            &part.tool_metadata_json,
            &part.tool_compacted_at,
            &part.tool_state_raw,
            &part.tool_state_time_start,
            &part.tool_state_time_end,
            &part.tool_state_time_compacted,
            &part.tool_attachments_json,
            &part.finish_reason,
            &part.cost,
            &part.token_total,
            &part.token_input,
            &part.token_output,
            &part.token_reasoning,
            &part.token_cache_read,
            &part.token_cache_write,
            &part.snapshot_hash,
            &part.patch_hash,
            &part.patch_files_json,
            &part.retry_attempt,
            &part.retry_error_json,
            &part.retry_created_at,
            &part.compaction_auto,
            &part.delta_field,
            &part.delta_text,
            &part.part_time_start,
            &part.part_time_end,
            &part.part_metadata_json,
            &part.text_synthetic,
            &part.text_ignored,
            &part.step_snapshot_hash,
            &part.created_at,
            &part.updated_at,
        ],
        row_to_assistant_message_part,
    )?;
    Ok(created)
}

pub fn update(
    conn: &Connection,
    part: &AssistantMessagePart,
) -> Result<AssistantMessagePart, DatabaseError> {
    let updated = conn
        .query_row(
            &format!(
                "UPDATE assistant_message_part
                 SET
                    harness_part_id = ?2,
                    assistant_message_id = ?3,
                    session_id = ?4,
                    position = ?5,
                    part_type = ?6,
                    text = ?7,
                    file_mime = ?8,
                    file_filename = ?9,
                    file_url = ?10,
                    file_source_type = ?11,
                    file_source_path = ?12,
                    file_source_name = ?13,
                    file_source_kind = ?14,
                    file_source_uri = ?15,
                    file_source_text_value = ?16,
                    file_source_text_start = ?17,
                    file_source_text_end = ?18,
                    agent_name = ?19,
                    subtask_prompt = ?20,
                    subtask_description = ?21,
                    subtask_agent = ?22,
                    subtask_model_provider_id = ?23,
                    subtask_model_id = ?24,
                    subtask_command = ?25,
                    tool_call_id = ?26,
                    tool_name = ?27,
                    tool_status = ?28,
                    tool_input_json = ?29,
                    tool_output_text = ?30,
                    tool_error_text = ?31,
                    tool_title = ?32,
                    tool_metadata_json = ?33,
                    tool_compacted_at = ?34,
                    tool_state_raw = ?35,
                    tool_state_time_start = ?36,
                    tool_state_time_end = ?37,
                    tool_state_time_compacted = ?38,
                    tool_attachments_json = ?39,
                    finish_reason = ?40,
                    cost = ?41,
                    token_total = ?42,
                    token_input = ?43,
                    token_output = ?44,
                    token_reasoning = ?45,
                    token_cache_read = ?46,
                    token_cache_write = ?47,
                    snapshot_hash = ?48,
                    patch_hash = ?49,
                    patch_files_json = ?50,
                    retry_attempt = ?51,
                    retry_error_json = ?52,
                    retry_created_at = ?53,
                    compaction_auto = ?54,
                    delta_field = ?55,
                    delta_text = ?56,
                    part_time_start = ?57,
                    part_time_end = ?58,
                    part_metadata_json = ?59,
                    text_synthetic = ?60,
                    text_ignored = ?61,
                    step_snapshot_hash = ?62,
                    updated_at = ?63
                 WHERE id = ?1
                 RETURNING {ASSISTANT_MESSAGE_PART_COLUMNS}"
            ),
            params![
                &part.id,
                &part.harness_part_id,
                &part.assistant_message_id,
                &part.session_id,
                &part.position,
                &part.part_type,
                &part.text,
                &part.file_mime,
                &part.file_filename,
                &part.file_url,
                &part.file_source_type,
                &part.file_source_path,
                &part.file_source_name,
                &part.file_source_kind,
                &part.file_source_uri,
                &part.file_source_text_value,
                &part.file_source_text_start,
                &part.file_source_text_end,
                &part.agent_name,
                &part.subtask_prompt,
                &part.subtask_description,
                &part.subtask_agent,
                &part.subtask_model_provider_id,
                &part.subtask_model_id,
                &part.subtask_command,
                &part.tool_call_id,
                &part.tool_name,
                &part.tool_status,
                &part.tool_input_json,
                &part.tool_output_text,
                &part.tool_error_text,
                &part.tool_title,
                &part.tool_metadata_json,
                &part.tool_compacted_at,
                &part.tool_state_raw,
                &part.tool_state_time_start,
                &part.tool_state_time_end,
                &part.tool_state_time_compacted,
                &part.tool_attachments_json,
                &part.finish_reason,
                &part.cost,
                &part.token_total,
                &part.token_input,
                &part.token_output,
                &part.token_reasoning,
                &part.token_cache_read,
                &part.token_cache_write,
                &part.snapshot_hash,
                &part.patch_hash,
                &part.patch_files_json,
                &part.retry_attempt,
                &part.retry_error_json,
                &part.retry_created_at,
                &part.compaction_auto,
                &part.delta_field,
                &part.delta_text,
                &part.part_time_start,
                &part.part_time_end,
                &part.part_metadata_json,
                &part.text_synthetic,
                &part.text_ignored,
                &part.step_snapshot_hash,
                now_utc_string(),
            ],
            row_to_assistant_message_part,
        )
        .map_err(|e| check_returning_row_error("update_assistant_message_part", e))?;
    Ok(updated)
}

pub fn delete(conn: &Connection, part_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM assistant_message_part WHERE id = ?1",
        [part_id],
    )?;
    assert_one_row_affected("delete_assistant_message_part", rows)
}
