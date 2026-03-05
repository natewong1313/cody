use serde_rusqlite::{from_rows, to_params_named, to_params_named_with_fields};
use tokio_rusqlite::named_params;
use tokio_rusqlite::rusqlite::Connection;
use uuid::Uuid;

use super::{assert_one_row_affected, expect_one_returned_row};
use crate::backend::{db::DatabaseError, repo::user_message::UserMessage};

pub const USER_MESSAGE_COLUMNS: &str = "
id, session_id, agent, model_provider_id, model_id, system_prompt,
structured_output_type, tools_list, thinking_variant, created_at, updated_at
";

pub fn get(conn: &Connection, user_message_id: Uuid) -> Result<Option<UserMessage>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {USER_MESSAGE_COLUMNS}
         FROM user_message
         WHERE id = :id"
    ))?;
    let mut rows = from_rows::<UserMessage>(stmt.query(named_params! {
        ":id": user_message_id.to_string()
    })?);
    Ok(rows.next().transpose()?)
}

pub fn list_by_session(
    conn: &Connection,
    session_id: Uuid,
    limit: u32,
) -> Result<Vec<UserMessage>, DatabaseError> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(&format!(
        "SELECT {USER_MESSAGE_COLUMNS}
         FROM user_message
         WHERE session_id = :session_id
         ORDER BY created_at DESC
         LIMIT :limit"
    ))?;

    let rows = from_rows::<UserMessage>(stmt.query(named_params! {
        ":session_id": session_id.to_string(),
        ":limit": i64::from(limit),
    })?);
    let mut rows = rows.collect::<Result<Vec<_>, _>>()?;
    rows.reverse();
    Ok(rows)
}

pub fn create(conn: &Connection, user_message: &UserMessage) -> Result<UserMessage, DatabaseError> {
    let params = to_params_named(user_message)?;
    let mut stmt = conn.prepare(&format!(
        "INSERT INTO user_message ({USER_MESSAGE_COLUMNS})
         VALUES (
             :id, :session_id, :agent, :model_provider_id, :model_id, :system_prompt,
             :structured_output_type, :tools_list, :thinking_variant, :created_at, :updated_at
         )
         RETURNING *"
    ))?;
    let rows = from_rows::<UserMessage>(stmt.query(params.to_slice().as_slice())?);
    expect_one_returned_row("create_user_message", rows)
}

pub fn update(conn: &Connection, user_message: &UserMessage) -> Result<UserMessage, DatabaseError> {
    let mut updated = user_message.clone();
    updated.updated_at = chrono::Utc::now().naive_utc();

    let params = to_params_named_with_fields(
        &updated,
        &[
            "id",
            "session_id",
            "agent",
            "model_provider_id",
            "model_id",
            "system_prompt",
            "structured_output_type",
            "tools_list",
            "thinking_variant",
            "updated_at",
        ],
    )?;
    let mut stmt = conn.prepare(
        "UPDATE user_message
         SET
            session_id = :session_id,
            agent = :agent,
            model_provider_id = :model_provider_id,
            model_id = :model_id,
            system_prompt = :system_prompt,
            structured_output_type = :structured_output_type,
            tools_list = :tools_list,
            thinking_variant = :thinking_variant,
            updated_at = :updated_at
         WHERE id = :id
         RETURNING *",
    )?;
    let rows = from_rows::<UserMessage>(stmt.query(params.to_slice().as_slice())?);
    expect_one_returned_row("update_user_message", rows)
}

pub fn delete(conn: &Connection, user_message_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM user_message WHERE id = :id",
        named_params! {":id": user_message_id.to_string()},
    )?;
    assert_one_row_affected("delete_user_message", rows)
}
