use tokio_rusqlite::rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{Message, db::DatabaseError};

const SELECT_MESSAGE_COLUMNS: &str = r#"
id, session_id, parent_message_id, role,
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
        session_id: row.get(1)?,
        parent_message_id: row.get(2)?,
        role: row.get(3)?,
        title: row.get(4)?,
        body: row.get(5)?,
        agent: row.get(6)?,
        system_message: row.get(7)?,
        variant: row.get(8)?,
        is_finished_streaming: row.get(9)?,
        is_summary: row.get(10)?,
        model_id: row.get(11)?,
        provider_id: row.get(12)?,
        error_name: row.get(13)?,
        error_message: row.get(14)?,
        error_type: row.get(15)?,
        cwd: row.get(16)?,
        root: row.get(17)?,
        cost: row.get(18)?,
        input_tokens: row.get(19)?,
        output_tokens: row.get(20)?,
        reasoning_tokens: row.get(21)?,
        cached_read_tokens: row.get(22)?,
        cached_write_tokens: row.get(23)?,
        total_tokens: row.get(24)?,
        completed_at: row.get(25)?,
        created_at: row.get(26)?,
        updated_at: row.get(27)?,
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
            id, session_id, parent_message_id, role,
            title, body, agent, system_message, variant,
            is_finished_streaming, is_summary,
            model_id, provider_id,
            error_name, error_message, error_type,
            cwd, root,
            cost, input_tokens, output_tokens, reasoning_tokens, cached_read_tokens, cached_write_tokens, total_tokens,
            completed_at, created_at, updated_at
        )
        VALUES (
            ?1, ?2, ?3, ?4,
            ?5, ?6, ?7, ?8, ?9,
            ?10, ?11,
            ?12, ?13,
            ?14, ?15, ?16,
            ?17, ?18,
            ?19, ?20, ?21, ?22, ?23, ?24, ?25,
            ?26, ?27, ?28
        )",
        params![
            &message.id,
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
                parent_message_id = ?3,
                role = ?4,
                title = ?5,
                body = ?6,
                agent = ?7,
                system_message = ?8,
                variant = ?9,
                is_finished_streaming = ?10,
                is_summary = ?11,
                model_id = ?12,
                provider_id = ?13,
                error_name = ?14,
                error_message = ?15,
                error_type = ?16,
                cwd = ?17,
                root = ?18,
                cost = ?19,
                input_tokens = ?20,
                output_tokens = ?21,
                reasoning_tokens = ?22,
                cached_read_tokens = ?23,
                cached_write_tokens = ?24,
                total_tokens = ?25,
                completed_at = ?26,
                updated_at = ?27
             WHERE id = ?1",
            params![
                &message.id,
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

pub fn delete_message(conn: &Connection, message_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM messages WHERE id = ?1", [message_id])?;
    assert_one_row_affected("delete_message", rows)
}
