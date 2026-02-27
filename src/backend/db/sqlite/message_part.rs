use tokio_rusqlite::rusqlite::{self, Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{MessagePart, db::DatabaseError};

const SELECT_PART_COLUMNS: &str = r#"
id, session_id, message_id, position, part_type,
text_content, synthetic, ignored, part_time_start, part_time_end,
mime, filename, url,
call_id, tool_name, tool_status, tool_title, tool_input_text, tool_output_text, tool_error_text, tool_time_start, tool_time_end, tool_time_compacted,
step_reason, step_snapshot, step_cost, step_input_tokens, step_output_tokens, step_reasoning_tokens, step_cached_read_tokens, step_cached_write_tokens, step_total_tokens,
subtask_prompt, subtask_description, subtask_agent, subtask_model_provider_id, subtask_model_id, subtask_command,
retry_attempt, retry_error_message, retry_error_status_code, retry_error_is_retryable,
snapshot_ref, patch_hash, compaction_auto, agent_name, agent_source_value, agent_source_start, agent_source_end,
created_at, updated_at
"#;

pub fn row_to_part(row: &Row) -> Result<MessagePart, rusqlite::Error> {
    Ok(MessagePart {
        id: row.get(0)?,
        session_id: row.get(1)?,
        message_id: row.get(2)?,
        position: row.get(3)?,
        part_type: row.get(4)?,
        text_content: row.get(5)?,
        synthetic: row.get(6)?,
        ignored: row.get(7)?,
        part_time_start: row.get(8)?,
        part_time_end: row.get(9)?,
        mime: row.get(10)?,
        filename: row.get(11)?,
        url: row.get(12)?,
        call_id: row.get(13)?,
        tool_name: row.get(14)?,
        tool_status: row.get(15)?,
        tool_title: row.get(16)?,
        tool_input_text: row.get(17)?,
        tool_output_text: row.get(18)?,
        tool_error_text: row.get(19)?,
        tool_time_start: row.get(20)?,
        tool_time_end: row.get(21)?,
        tool_time_compacted: row.get(22)?,
        step_reason: row.get(23)?,
        step_snapshot: row.get(24)?,
        step_cost: row.get(25)?,
        step_input_tokens: row.get(26)?,
        step_output_tokens: row.get(27)?,
        step_reasoning_tokens: row.get(28)?,
        step_cached_read_tokens: row.get(29)?,
        step_cached_write_tokens: row.get(30)?,
        step_total_tokens: row.get(31)?,
        subtask_prompt: row.get(32)?,
        subtask_description: row.get(33)?,
        subtask_agent: row.get(34)?,
        subtask_model_provider_id: row.get(35)?,
        subtask_model_id: row.get(36)?,
        subtask_command: row.get(37)?,
        retry_attempt: row.get(38)?,
        retry_error_message: row.get(39)?,
        retry_error_status_code: row.get(40)?,
        retry_error_is_retryable: row.get(41)?,
        snapshot_ref: row.get(42)?,
        patch_hash: row.get(43)?,
        compaction_auto: row.get(44)?,
        agent_name: row.get(45)?,
        agent_source_value: row.get(46)?,
        agent_source_start: row.get(47)?,
        agent_source_end: row.get(48)?,
        created_at: row.get(49)?,
        updated_at: row.get(50)?,
    })
}

pub fn list_parts_by_message(
    conn: &Connection,
    message_id: Uuid,
) -> Result<Vec<MessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_PART_COLUMNS}
         FROM message_parts
         WHERE message_id = ?1
         ORDER BY position ASC, created_at ASC"
    ))?;

    let parts = stmt
        .query_map([message_id], row_to_part)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts)
}

pub fn get_part(conn: &Connection, part_id: Uuid) -> Result<Option<MessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_PART_COLUMNS}
         FROM message_parts
         WHERE id = ?1"
    ))?;
    let part = stmt.query_row([part_id], row_to_part).optional()?;
    Ok(part)
}

pub fn create_part(conn: &Connection, part: &MessagePart) -> Result<MessagePart, DatabaseError> {
    let rows = conn.execute(
        "INSERT INTO message_parts (
            id, session_id, message_id, position, part_type,
            text_content, synthetic, ignored, part_time_start, part_time_end,
            mime, filename, url,
            call_id, tool_name, tool_status, tool_title, tool_input_text, tool_output_text, tool_error_text, tool_time_start, tool_time_end, tool_time_compacted,
            step_reason, step_snapshot, step_cost, step_input_tokens, step_output_tokens, step_reasoning_tokens, step_cached_read_tokens, step_cached_write_tokens, step_total_tokens,
            subtask_prompt, subtask_description, subtask_agent, subtask_model_provider_id, subtask_model_id, subtask_command,
            retry_attempt, retry_error_message, retry_error_status_code, retry_error_is_retryable,
            snapshot_ref, patch_hash, compaction_auto, agent_name, agent_source_value, agent_source_start, agent_source_end,
            created_at, updated_at
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13,
            ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23,
            ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32,
            ?33, ?34, ?35, ?36, ?37, ?38,
            ?39, ?40, ?41, ?42,
            ?43, ?44, ?45, ?46, ?47, ?48, ?49,
            ?50, ?51
        )",
        params![
            &part.id,
            &part.session_id,
            &part.message_id,
            &part.position,
            &part.part_type,
            &part.text_content,
            &part.synthetic,
            &part.ignored,
            &part.part_time_start,
            &part.part_time_end,
            &part.mime,
            &part.filename,
            &part.url,
            &part.call_id,
            &part.tool_name,
            &part.tool_status,
            &part.tool_title,
            &part.tool_input_text,
            &part.tool_output_text,
            &part.tool_error_text,
            &part.tool_time_start,
            &part.tool_time_end,
            &part.tool_time_compacted,
            &part.step_reason,
            &part.step_snapshot,
            &part.step_cost,
            &part.step_input_tokens,
            &part.step_output_tokens,
            &part.step_reasoning_tokens,
            &part.step_cached_read_tokens,
            &part.step_cached_write_tokens,
            &part.step_total_tokens,
            &part.subtask_prompt,
            &part.subtask_description,
            &part.subtask_agent,
            &part.subtask_model_provider_id,
            &part.subtask_model_id,
            &part.subtask_command,
            &part.retry_attempt,
            &part.retry_error_message,
            &part.retry_error_status_code,
            &part.retry_error_is_retryable,
            &part.snapshot_ref,
            &part.patch_hash,
            &part.compaction_auto,
            &part.agent_name,
            &part.agent_source_value,
            &part.agent_source_start,
            &part.agent_source_end,
            &part.created_at,
            &part.updated_at,
        ],
    )?;
    assert_one_row_affected("create_part", rows)?;
    get_part(conn, part.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "create_part",
        expected: 1,
        actual: 0,
    })
}

pub fn update_part(conn: &Connection, part: &MessagePart) -> Result<MessagePart, DatabaseError> {
    let rows = conn
        .execute(
            "UPDATE message_parts
             SET
                session_id = ?2,
                message_id = ?3,
                position = ?4,
                part_type = ?5,
                text_content = ?6,
                synthetic = ?7,
                ignored = ?8,
                part_time_start = ?9,
                part_time_end = ?10,
                mime = ?11,
                filename = ?12,
                url = ?13,
                call_id = ?14,
                tool_name = ?15,
                tool_status = ?16,
                tool_title = ?17,
                tool_input_text = ?18,
                tool_output_text = ?19,
                tool_error_text = ?20,
                tool_time_start = ?21,
                tool_time_end = ?22,
                tool_time_compacted = ?23,
                step_reason = ?24,
                step_snapshot = ?25,
                step_cost = ?26,
                step_input_tokens = ?27,
                step_output_tokens = ?28,
                step_reasoning_tokens = ?29,
                step_cached_read_tokens = ?30,
                step_cached_write_tokens = ?31,
                step_total_tokens = ?32,
                subtask_prompt = ?33,
                subtask_description = ?34,
                subtask_agent = ?35,
                subtask_model_provider_id = ?36,
                subtask_model_id = ?37,
                subtask_command = ?38,
                retry_attempt = ?39,
                retry_error_message = ?40,
                retry_error_status_code = ?41,
                retry_error_is_retryable = ?42,
                snapshot_ref = ?43,
                patch_hash = ?44,
                compaction_auto = ?45,
                agent_name = ?46,
                agent_source_value = ?47,
                agent_source_start = ?48,
                agent_source_end = ?49,
                updated_at = ?50
             WHERE id = ?1",
            params![
                &part.id,
                &part.session_id,
                &part.message_id,
                &part.position,
                &part.part_type,
                &part.text_content,
                &part.synthetic,
                &part.ignored,
                &part.part_time_start,
                &part.part_time_end,
                &part.mime,
                &part.filename,
                &part.url,
                &part.call_id,
                &part.tool_name,
                &part.tool_status,
                &part.tool_title,
                &part.tool_input_text,
                &part.tool_output_text,
                &part.tool_error_text,
                &part.tool_time_start,
                &part.tool_time_end,
                &part.tool_time_compacted,
                &part.step_reason,
                &part.step_snapshot,
                &part.step_cost,
                &part.step_input_tokens,
                &part.step_output_tokens,
                &part.step_reasoning_tokens,
                &part.step_cached_read_tokens,
                &part.step_cached_write_tokens,
                &part.step_total_tokens,
                &part.subtask_prompt,
                &part.subtask_description,
                &part.subtask_agent,
                &part.subtask_model_provider_id,
                &part.subtask_model_id,
                &part.subtask_command,
                &part.retry_attempt,
                &part.retry_error_message,
                &part.retry_error_status_code,
                &part.retry_error_is_retryable,
                &part.snapshot_ref,
                &part.patch_hash,
                &part.compaction_auto,
                &part.agent_name,
                &part.agent_source_value,
                &part.agent_source_start,
                &part.agent_source_end,
                now_utc_string(),
            ],
        )
        .map_err(|e| check_returning_row_error("update_part", e))?;
    assert_one_row_affected("update_part", rows)?;
    get_part(conn, part.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "update_part",
        expected: 1,
        actual: 0,
    })
}

pub fn delete_part(conn: &Connection, part_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM message_parts WHERE id = ?1", [part_id])?;
    assert_one_row_affected("delete_part", rows)
}
