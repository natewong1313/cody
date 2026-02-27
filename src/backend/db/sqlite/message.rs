use chrono::NaiveDateTime;
use tokio_rusqlite::rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{Message, MessageTool, db::DatabaseError};

const SELECT_MESSAGE_COLUMNS: &str = r#"
id, harness_message_id, session_id, parent_message_id, role,
title, body, agent, system_message, variant,
is_finished_streaming, is_summary,
model_id, provider_id,
error_name, error_message, error_type,
cwd, root,
cost, input_tokens, output_tokens, reasoning_tokens, cached_read_tokens, cached_write_tokens, total_tokens,
completed_at, created_at, updated_at
"#;

pub fn row_to_message(row: &Row) -> Result<Message, tokio_rusqlite::rusqlite::Error> {
    Ok(Message {
        id: row.get(0)?,
        harness_message_id: row.get(1)?,
        session_id: row.get(2)?,
        parent_message_id: row.get(3)?,
        role: row.get(4)?,
        title: row.get(5)?,
        body: row.get(6)?,
        agent: row.get(7)?,
        system_message: row.get(8)?,
        variant: row.get(9)?,
        is_finished_streaming: row.get(10)?,
        is_summary: row.get(11)?,
        model_id: row.get(12)?,
        provider_id: row.get(13)?,
        error_name: row.get(14)?,
        error_message: row.get(15)?,
        error_type: row.get(16)?,
        cwd: row.get(17)?,
        root: row.get(18)?,
        cost: row.get(19)?,
        input_tokens: row.get(20)?,
        output_tokens: row.get(21)?,
        reasoning_tokens: row.get(22)?,
        cached_read_tokens: row.get(23)?,
        cached_write_tokens: row.get(24)?,
        total_tokens: row.get(25)?,
        completed_at: row.get(26)?,
        created_at: row.get(27)?,
        updated_at: row.get(28)?,
    })
}

pub fn list_messages_by_session(
    conn: &Connection,
    session_id: Uuid,
) -> Result<Vec<Message>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_MESSAGE_COLUMNS}
         FROM messages
         WHERE session_id = ?1
         ORDER BY created_at ASC"
    ))?;

    let messages = stmt
        .query_map([session_id], row_to_message)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(messages)
}

pub fn get_message(conn: &Connection, message_id: Uuid) -> Result<Option<Message>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_MESSAGE_COLUMNS}
         FROM messages
         WHERE id = ?1"
    ))?;
    let message = stmt.query_row([message_id], row_to_message).optional()?;
    Ok(message)
}

pub fn create_message(conn: &Connection, message: &Message) -> Result<Message, DatabaseError> {
    let rows = conn.execute(
        "INSERT INTO messages (
            id, harness_message_id, session_id, parent_message_id, role,
            title, body, agent, system_message, variant,
            is_finished_streaming, is_summary,
            model_id, provider_id,
            error_name, error_message, error_type,
            cwd, root,
            cost, input_tokens, output_tokens, reasoning_tokens, cached_read_tokens, cached_write_tokens, total_tokens,
            completed_at, created_at, updated_at
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10,
            ?11, ?12,
            ?13, ?14,
            ?15, ?16, ?17,
            ?18, ?19,
            ?20, ?21, ?22, ?23, ?24, ?25, ?26,
            ?27, ?28, ?29
        )",
        params![
            &message.id,
            &message.harness_message_id,
            &message.session_id,
            &message.parent_message_id,
            &message.role,
            &message.title,
            &message.body,
            &message.agent,
            &message.system_message,
            &message.variant,
            &message.is_finished_streaming,
            &message.is_summary,
            &message.model_id,
            &message.provider_id,
            &message.error_name,
            &message.error_message,
            &message.error_type,
            &message.cwd,
            &message.root,
            &message.cost,
            &message.input_tokens,
            &message.output_tokens,
            &message.reasoning_tokens,
            &message.cached_read_tokens,
            &message.cached_write_tokens,
            &message.total_tokens,
            &message.completed_at,
            &message.created_at,
            &message.updated_at,
        ],
    )?;
    assert_one_row_affected("create_message", rows)?;
    get_message(conn, message.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "create_message",
        expected: 1,
        actual: 0,
    })
}

pub fn update_message(conn: &Connection, message: &Message) -> Result<Message, DatabaseError> {
    let rows = conn
        .execute(
            "UPDATE messages
             SET
                session_id = ?2,
                harness_message_id = ?3,
                parent_message_id = ?4,
                role = ?5,
                title = ?6,
                body = ?7,
                agent = ?8,
                system_message = ?9,
                variant = ?10,
                is_finished_streaming = ?11,
                is_summary = ?12,
                model_id = ?13,
                provider_id = ?14,
                error_name = ?15,
                error_message = ?16,
                error_type = ?17,
                cwd = ?18,
                root = ?19,
                cost = ?20,
                input_tokens = ?21,
                output_tokens = ?22,
                reasoning_tokens = ?23,
                cached_read_tokens = ?24,
                cached_write_tokens = ?25,
                total_tokens = ?26,
                completed_at = ?27,
                updated_at = ?28
             WHERE id = ?1",
            params![
                &message.id,
                &message.session_id,
                &message.harness_message_id,
                &message.parent_message_id,
                &message.role,
                &message.title,
                &message.body,
                &message.agent,
                &message.system_message,
                &message.variant,
                &message.is_finished_streaming,
                &message.is_summary,
                &message.model_id,
                &message.provider_id,
                &message.error_name,
                &message.error_message,
                &message.error_type,
                &message.cwd,
                &message.root,
                &message.cost,
                &message.input_tokens,
                &message.output_tokens,
                &message.reasoning_tokens,
                &message.cached_read_tokens,
                &message.cached_write_tokens,
                &message.total_tokens,
                &message.completed_at,
                now_utc_string(),
            ],
        )
        .map_err(|e| check_returning_row_error("update_message", e))?;
    assert_one_row_affected("update_message", rows)?;
    get_message(conn, message.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "update_message",
        expected: 1,
        actual: 0,
    })
}

pub fn get_message_by_harness_message_id(
    conn: &Connection,
    session_id: Uuid,
    harness_message_id: &str,
) -> Result<Option<Message>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_MESSAGE_COLUMNS}
         FROM messages
         WHERE session_id = ?1 AND harness_message_id = ?2"
    ))?;
    let message = stmt
        .query_row(params![session_id, harness_message_id], row_to_message)
        .optional()?;
    Ok(message)
}

pub fn delete_message_by_harness_message_id(
    conn: &Connection,
    session_id: Uuid,
    harness_message_id: &str,
) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM messages WHERE session_id = ?1 AND harness_message_id = ?2",
        params![session_id, harness_message_id],
    )?;
    assert_one_row_affected("delete_message_by_harness_message_id", rows)
}

pub fn mark_session_assistant_messages_finished(
    conn: &Connection,
    session_id: Uuid,
    completed_at: NaiveDateTime,
) -> Result<(), DatabaseError> {
    conn.execute(
        "UPDATE messages
         SET is_finished_streaming = 1, completed_at = COALESCE(completed_at, ?2), updated_at = ?3
         WHERE session_id = ?1 AND role = 'assistant' AND is_finished_streaming = 0",
        params![session_id, completed_at, now_utc_string()],
    )?;
    Ok(())
}

pub fn delete_message(conn: &Connection, message_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM messages WHERE id = ?1", [message_id])?;
    assert_one_row_affected("delete_message", rows)
}

pub fn row_to_message_tool(row: &Row) -> Result<MessageTool, tokio_rusqlite::rusqlite::Error> {
    Ok(MessageTool {
        message_id: row.get(0)?,
        tool_name: row.get(1)?,
        enabled: row.get(2)?,
    })
}

pub fn list_message_tools(
    conn: &Connection,
    message_id: Uuid,
) -> Result<Vec<MessageTool>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT message_id, tool_name, enabled
         FROM message_tools
         WHERE message_id = ?1
         ORDER BY tool_name ASC",
    )?;

    let tools = stmt
        .query_map([message_id], row_to_message_tool)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tools)
}

pub fn upsert_message_tool(
    conn: &Connection,
    tool: &MessageTool,
) -> Result<MessageTool, DatabaseError> {
    let rows = conn.execute(
        "INSERT INTO message_tools (message_id, tool_name, enabled)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(message_id, tool_name)
         DO UPDATE SET enabled = excluded.enabled",
        params![&tool.message_id, &tool.tool_name, &tool.enabled],
    )?;
    assert_one_row_affected("upsert_message_tool", rows)?;

    let mut stmt = conn.prepare(
        "SELECT message_id, tool_name, enabled
         FROM message_tools
         WHERE message_id = ?1 AND tool_name = ?2",
    )?;
    let out = stmt.query_row(
        params![&tool.message_id, &tool.tool_name],
        row_to_message_tool,
    )?;
    Ok(out)
}

pub fn delete_message_tool(
    conn: &Connection,
    message_id: Uuid,
    tool_name: &str,
) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM message_tools WHERE message_id = ?1 AND tool_name = ?2",
        params![message_id, tool_name],
    )?;
    assert_one_row_affected("delete_message_tool", rows)
}
