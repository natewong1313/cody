use tokio_rusqlite::rusqlite::{self, Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{
    db::DatabaseError,
    repo::assistant_message::{AssistantMessage, AssistantMessagePart},
};

const ASSISTANT_MESSAGE_COLUMNS: &str = "
id, session_id, user_message_id, agent, model_provider_id, model_id,
cwd, root, cost,
token_total, token_input, token_output, token_reasoning, token_cache_read, token_cache_write,
error_message, created_at, updated_at, completed_at
";

const ASSISTANT_MESSAGE_PART_COLUMNS: &str = "
id, assistant_message_id, session_id, position, part_type, text,
file_mime, file_filename, file_url,
file_source_type, file_source_path, file_source_name, file_source_kind, file_source_uri,
file_source_text_value, file_source_text_start, file_source_text_end,
agent_name, subtask_prompt, subtask_description, subtask_agent,
subtask_model_provider_id, subtask_model_id, subtask_command,
tool_call_id, tool_name, tool_status, tool_input_json, tool_output_text, tool_error_text,
tool_title, tool_metadata_json, tool_compacted_at,
finish_reason, cost, token_total, token_input, token_output, token_reasoning,
token_cache_read, token_cache_write,
snapshot_hash, patch_hash, patch_files_json, retry_attempt, retry_error_json, retry_created_at,
compaction_auto, created_at, updated_at
";

pub fn row_to_assistant_message(row: &Row) -> Result<AssistantMessage, rusqlite::Error> {
    Ok(AssistantMessage {
        id: row.get(0)?,
        session_id: row.get(1)?,
        user_message_id: row.get(2)?,
        agent: row.get(3)?,
        model_provider_id: row.get(4)?,
        model_id: row.get(5)?,
        cwd: row.get(6)?,
        root: row.get(7)?,
        cost: row.get(8)?,
        token_total: row.get(9)?,
        token_input: row.get(10)?,
        token_output: row.get(11)?,
        token_reasoning: row.get(12)?,
        token_cache_read: row.get(13)?,
        token_cache_write: row.get(14)?,
        error_message: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        completed_at: row.get(18)?,
    })
}

pub fn get_assistant_message(
    conn: &Connection,
    assistant_message_id: Uuid,
) -> Result<Option<AssistantMessage>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ASSISTANT_MESSAGE_COLUMNS}
         FROM assistant_message
         WHERE id = ?1"
    ))?;
    Ok(stmt
        .query_row([assistant_message_id], row_to_assistant_message)
        .optional()?)
}

pub fn create_assistant_message(
    conn: &Connection,
    assistant_message: &AssistantMessage,
) -> Result<AssistantMessage, DatabaseError> {
    let rows = conn.execute(
        &format!(
            "INSERT INTO assistant_message ({ASSISTANT_MESSAGE_COLUMNS})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)"
        ),
        params![
            &assistant_message.id,
            &assistant_message.session_id,
            &assistant_message.user_message_id,
            &assistant_message.agent,
            &assistant_message.model_provider_id,
            &assistant_message.model_id,
            &assistant_message.cwd,
            &assistant_message.root,
            &assistant_message.cost,
            &assistant_message.token_total,
            &assistant_message.token_input,
            &assistant_message.token_output,
            &assistant_message.token_reasoning,
            &assistant_message.token_cache_read,
            &assistant_message.token_cache_write,
            &assistant_message.error_message,
            &assistant_message.created_at,
            &assistant_message.updated_at,
            &assistant_message.completed_at,
        ],
    )?;
    assert_one_row_affected("create_assistant_message", rows)?;
    get_assistant_message(conn, assistant_message.id)?.ok_or(
        DatabaseError::UnexpectedRowsAffected {
            op: "create_assistant_message",
            expected: 1,
            actual: 0,
        },
    )
}

pub fn update_assistant_message(
    conn: &Connection,
    assistant_message: &AssistantMessage,
) -> Result<AssistantMessage, DatabaseError> {
    let rows = conn
        .execute(
            "UPDATE assistant_message
             SET
                session_id = ?2,
                user_message_id = ?3,
                agent = ?4,
                model_provider_id = ?5,
                model_id = ?6,
                cwd = ?7,
                root = ?8,
                cost = ?9,
                token_total = ?10,
                token_input = ?11,
                token_output = ?12,
                token_reasoning = ?13,
                token_cache_read = ?14,
                token_cache_write = ?15,
                error_message = ?16,
                updated_at = ?17,
                completed_at = ?18
             WHERE id = ?1",
            params![
                &assistant_message.id,
                &assistant_message.session_id,
                &assistant_message.user_message_id,
                &assistant_message.agent,
                &assistant_message.model_provider_id,
                &assistant_message.model_id,
                &assistant_message.cwd,
                &assistant_message.root,
                &assistant_message.cost,
                &assistant_message.token_total,
                &assistant_message.token_input,
                &assistant_message.token_output,
                &assistant_message.token_reasoning,
                &assistant_message.token_cache_read,
                &assistant_message.token_cache_write,
                &assistant_message.error_message,
                now_utc_string(),
                &assistant_message.completed_at,
            ],
        )
        .map_err(|e| check_returning_row_error("update_assistant_message", e))?;
    assert_one_row_affected("update_assistant_message", rows)?;
    get_assistant_message(conn, assistant_message.id)?.ok_or(
        DatabaseError::UnexpectedRowsAffected {
            op: "update_assistant_message",
            expected: 1,
            actual: 0,
        },
    )
}

pub fn delete_assistant_message(
    conn: &Connection,
    assistant_message_id: Uuid,
) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM assistant_message WHERE id = ?1",
        [assistant_message_id],
    )?;
    assert_one_row_affected("delete_assistant_message", rows)
}

pub fn row_to_assistant_message_part(row: &Row) -> Result<AssistantMessagePart, rusqlite::Error> {
    Ok(AssistantMessagePart {
        id: row.get(0)?,
        assistant_message_id: row.get(1)?,
        session_id: row.get(2)?,
        position: row.get(3)?,
        part_type: row.get(4)?,
        text: row.get(5)?,
        file_mime: row.get(6)?,
        file_filename: row.get(7)?,
        file_url: row.get(8)?,
        file_source_type: row.get(9)?,
        file_source_path: row.get(10)?,
        file_source_name: row.get(11)?,
        file_source_kind: row.get(12)?,
        file_source_uri: row.get(13)?,
        file_source_text_value: row.get(14)?,
        file_source_text_start: row.get(15)?,
        file_source_text_end: row.get(16)?,
        agent_name: row.get(17)?,
        subtask_prompt: row.get(18)?,
        subtask_description: row.get(19)?,
        subtask_agent: row.get(20)?,
        subtask_model_provider_id: row.get(21)?,
        subtask_model_id: row.get(22)?,
        subtask_command: row.get(23)?,
        tool_call_id: row.get(24)?,
        tool_name: row.get(25)?,
        tool_status: row.get(26)?,
        tool_input_json: row.get(27)?,
        tool_output_text: row.get(28)?,
        tool_error_text: row.get(29)?,
        tool_title: row.get(30)?,
        tool_metadata_json: row.get(31)?,
        tool_compacted_at: row.get(32)?,
        finish_reason: row.get(33)?,
        cost: row.get(34)?,
        token_total: row.get(35)?,
        token_input: row.get(36)?,
        token_output: row.get(37)?,
        token_reasoning: row.get(38)?,
        token_cache_read: row.get(39)?,
        token_cache_write: row.get(40)?,
        snapshot_hash: row.get(41)?,
        patch_hash: row.get(42)?,
        patch_files_json: row.get(43)?,
        retry_attempt: row.get(44)?,
        retry_error_json: row.get(45)?,
        retry_created_at: row.get(46)?,
        compaction_auto: row.get(47)?,
        created_at: row.get(48)?,
        updated_at: row.get(49)?,
    })
}

pub fn get_assistant_message_part(
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

pub fn create_assistant_message_part(
    conn: &Connection,
    part: &AssistantMessagePart,
) -> Result<AssistantMessagePart, DatabaseError> {
    let rows = conn.execute(
        &format!(
            "INSERT INTO assistant_message_part ({ASSISTANT_MESSAGE_PART_COLUMNS})
             VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                ?18, ?19, ?20, ?21, ?22, ?23, ?24,
                ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33,
                ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41,
                ?42, ?43, ?44, ?45, ?46, ?47,
                ?48, ?49, ?50
             )"
        ),
        params![
            &part.id,
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
            &part.created_at,
            &part.updated_at,
        ],
    )?;
    assert_one_row_affected("create_assistant_message_part", rows)?;
    get_assistant_message_part(conn, part.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "create_assistant_message_part",
        expected: 1,
        actual: 0,
    })
}

pub fn update_assistant_message_part(
    conn: &Connection,
    part: &AssistantMessagePart,
) -> Result<AssistantMessagePart, DatabaseError> {
    let rows = conn
        .execute(
            "UPDATE assistant_message_part
             SET
                assistant_message_id = ?2,
                session_id = ?3,
                position = ?4,
                part_type = ?5,
                text = ?6,
                file_mime = ?7,
                file_filename = ?8,
                file_url = ?9,
                file_source_type = ?10,
                file_source_path = ?11,
                file_source_name = ?12,
                file_source_kind = ?13,
                file_source_uri = ?14,
                file_source_text_value = ?15,
                file_source_text_start = ?16,
                file_source_text_end = ?17,
                agent_name = ?18,
                subtask_prompt = ?19,
                subtask_description = ?20,
                subtask_agent = ?21,
                subtask_model_provider_id = ?22,
                subtask_model_id = ?23,
                subtask_command = ?24,
                tool_call_id = ?25,
                tool_name = ?26,
                tool_status = ?27,
                tool_input_json = ?28,
                tool_output_text = ?29,
                tool_error_text = ?30,
                tool_title = ?31,
                tool_metadata_json = ?32,
                tool_compacted_at = ?33,
                finish_reason = ?34,
                cost = ?35,
                token_total = ?36,
                token_input = ?37,
                token_output = ?38,
                token_reasoning = ?39,
                token_cache_read = ?40,
                token_cache_write = ?41,
                snapshot_hash = ?42,
                patch_hash = ?43,
                patch_files_json = ?44,
                retry_attempt = ?45,
                retry_error_json = ?46,
                retry_created_at = ?47,
                compaction_auto = ?48,
                updated_at = ?49
             WHERE id = ?1",
            params![
                &part.id,
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
                now_utc_string(),
            ],
        )
        .map_err(|e| check_returning_row_error("update_assistant_message_part", e))?;
    assert_one_row_affected("update_assistant_message_part", rows)?;
    get_assistant_message_part(conn, part.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "update_assistant_message_part",
        expected: 1,
        actual: 0,
    })
}

pub fn delete_assistant_message_part(
    conn: &Connection,
    part_id: Uuid,
) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM assistant_message_part WHERE id = ?1",
        [part_id],
    )?;
    assert_one_row_affected("delete_assistant_message_part", rows)
}
