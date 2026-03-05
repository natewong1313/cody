use serde_rusqlite::{from_rows, to_params_named, to_params_named_with_fields};
use tokio_rusqlite::named_params;
use tokio_rusqlite::rusqlite::Connection;
use uuid::Uuid;

use crate::backend::{db::DatabaseError, repo::assistant_message::AssistantMessage};

pub const ASSISTANT_MESSAGE_COLUMNS: &str = "
id, harness_message_id, session_id, user_message_id, agent, model_provider_id, model_id,
cwd, root, cost,
token_total, token_input, token_output, token_reasoning, token_cache_read, token_cache_write,
error_message, created_at, updated_at, completed_at
";

pub fn get(
    conn: &Connection,
    assistant_message_id: Uuid,
) -> Result<Option<AssistantMessage>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT * FROM assistant_message WHERE id = :id")?;
    let mut rows = from_rows::<AssistantMessage>(
        stmt.query(named_params! {":id": assistant_message_id.to_string()})?,
    );
    Ok(rows.next().transpose()?)
}

pub fn get_by_harness_id(
    conn: &Connection,
    session_id: Uuid,
    harness_message_id: &str,
) -> Result<Option<AssistantMessage>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM assistant_message WHERE session_id = :session_id AND harness_message_id = :harness_message_id",
    )?;
    let mut rows = from_rows::<AssistantMessage>(stmt.query(named_params! {
        ":session_id": session_id.to_string(),
        ":harness_message_id": harness_message_id,
    })?);
    Ok(rows.next().transpose()?)
}

pub fn create(
    conn: &Connection,
    assistant_message: &AssistantMessage,
) -> Result<AssistantMessage, DatabaseError> {
    let params = to_params_named(assistant_message)?;
    let mut stmt = conn.prepare(
        &format!("
        INSERT INTO assistant_message ({ASSISTANT_MESSAGE_COLUMNS})
        VALUES (
            :id, :harness_message_id, :session_id, :user_message_id, :agent, :model_provider_id, :model_id,
            :cwd, :root, :cost,
            :token_total, :token_input, :token_output, :token_reasoning, :token_cache_read, :token_cache_write,
            :error_message, :created_at, :updated_at, :completed_at
        )
        RETURNING *
    "),
    )?;
    let rows = from_rows::<AssistantMessage>(stmt.query(params.to_slice().as_slice())?);
    super::expect_one_returned_row("create_assistant_message", rows)
}

pub fn update(
    conn: &Connection,
    assistant_message: &AssistantMessage,
) -> Result<AssistantMessage, DatabaseError> {
    let mut updated = assistant_message.clone();
    updated.updated_at = chrono::Utc::now().naive_utc();

    let params = to_params_named_with_fields(
        &updated,
        &[
            "id",
            "harness_message_id",
            "session_id",
            "user_message_id",
            "agent",
            "model_provider_id",
            "model_id",
            "cwd",
            "root",
            "cost",
            "token_total",
            "token_input",
            "token_output",
            "token_reasoning",
            "token_cache_read",
            "token_cache_write",
            "error_message",
            "updated_at",
            "completed_at",
        ],
    )?;
    let mut stmt = conn.prepare(
        "
        UPDATE assistant_message
        SET
            harness_message_id = :harness_message_id,
            session_id = :session_id,
            user_message_id = :user_message_id,
            agent = :agent,
            model_provider_id = :model_provider_id,
            model_id = :model_id,
            cwd = :cwd,
            root = :root,
            cost = :cost,
            token_total = :token_total,
            token_input = :token_input,
            token_output = :token_output,
            token_reasoning = :token_reasoning,
            token_cache_read = :token_cache_read,
            token_cache_write = :token_cache_write,
            error_message = :error_message,
            updated_at = :updated_at,
            completed_at = :completed_at
        WHERE id = :id
        RETURNING *
    ",
    )?;
    let rows = from_rows::<AssistantMessage>(stmt.query(params.to_slice().as_slice())?);
    super::expect_one_returned_row("update_assistant_message", rows)
}

pub fn delete(conn: &Connection, assistant_message_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM assistant_message WHERE id = :id",
        named_params! {":id": assistant_message_id.to_string()},
    )?;
    super::assert_one_row_affected("delete_assistant_message", rows)
}
