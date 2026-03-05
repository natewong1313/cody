use tokio_rusqlite::rusqlite::{self, Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{db::DatabaseError, repo::assistant_message::AssistantMessage};

pub const ASSISTANT_MESSAGE_COLUMNS: &str = "
id, harness_message_id, session_id, user_message_id, agent, model_provider_id, model_id,
cwd, root, cost,
token_total, token_input, token_output, token_reasoning, token_cache_read, token_cache_write,
error_message, created_at, updated_at, completed_at
";

pub const ASSISTANT_MESSAGE_COLUMN_COUNT: usize = 20;

pub fn row_to_assistant_message(row: &Row) -> Result<AssistantMessage, rusqlite::Error> {
    row_to_assistant_message_at(row, 0)
}

pub fn row_to_assistant_message_at(
    row: &Row,
    start: usize,
) -> Result<AssistantMessage, rusqlite::Error> {
    Ok(AssistantMessage {
        id: row.get(start)?,
        harness_message_id: row.get(start + 1)?,
        session_id: row.get(start + 2)?,
        user_message_id: row.get(start + 3)?,
        agent: row.get(start + 4)?,
        model_provider_id: row.get(start + 5)?,
        model_id: row.get(start + 6)?,
        cwd: row.get(start + 7)?,
        root: row.get(start + 8)?,
        cost: row.get(start + 9)?,
        token_total: row.get(start + 10)?,
        token_input: row.get(start + 11)?,
        token_output: row.get(start + 12)?,
        token_reasoning: row.get(start + 13)?,
        token_cache_read: row.get(start + 14)?,
        token_cache_write: row.get(start + 15)?,
        error_message: row.get(start + 16)?,
        created_at: row.get(start + 17)?,
        updated_at: row.get(start + 18)?,
        completed_at: row.get(start + 19)?,
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

pub fn get_assistant_message_by_harness_id(
    conn: &Connection,
    session_id: Uuid,
    harness_message_id: &str,
) -> Result<Option<AssistantMessage>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ASSISTANT_MESSAGE_COLUMNS}
         FROM assistant_message
         WHERE session_id = ?1 AND harness_message_id = ?2"
    ))?;
    Ok(stmt
        .query_row(
            params![session_id, harness_message_id],
            row_to_assistant_message,
        )
        .optional()?)
}

pub fn create_assistant_message(
    conn: &Connection,
    assistant_message: &AssistantMessage,
) -> Result<AssistantMessage, DatabaseError> {
    let rows = conn.execute(
        &format!(
            "INSERT INTO assistant_message ({ASSISTANT_MESSAGE_COLUMNS})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)"
        ),
        params![
            &assistant_message.id,
            &assistant_message.harness_message_id,
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
                harness_message_id = ?2,
                session_id = ?3,
                user_message_id = ?4,
                agent = ?5,
                model_provider_id = ?6,
                model_id = ?7,
                cwd = ?8,
                root = ?9,
                cost = ?10,
                token_total = ?11,
                token_input = ?12,
                token_output = ?13,
                token_reasoning = ?14,
                token_cache_read = ?15,
                token_cache_write = ?16,
                error_message = ?17,
                updated_at = ?18,
                completed_at = ?19
             WHERE id = ?1",
            params![
                &assistant_message.id,
                &assistant_message.harness_message_id,
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
