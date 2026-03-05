use serde_rusqlite::{from_rows, to_params_named, to_params_named_with_fields};
use tokio_rusqlite::named_params;
use tokio_rusqlite::rusqlite::Connection;
use uuid::Uuid;

use crate::backend::{
    db::{
        sqlite::{assert_one_row_affected, expect_one_returned_row},
        DatabaseError,
    },
    repo::user_message_part::UserMessagePart,
};

const USER_MESSAGE_PART_COLUMNS: &str = "
id, user_message_id, session_id, position, part_type,
text, file_name, file_url, agent_name, subtask_prompt, subtask_description,
created_at, updated_at
";

pub fn get(conn: &Connection, part_id: Uuid) -> Result<Option<UserMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT *
         FROM user_message_part
         WHERE id = :id"
    ))?;
    let mut rows = from_rows::<UserMessagePart>(stmt.query(named_params! {
        ":id": part_id.to_string()
    })?);
    Ok(rows.next().transpose()?)
}

pub fn create(conn: &Connection, part: &UserMessagePart) -> Result<UserMessagePart, DatabaseError> {
    let params = to_params_named(part)?;
    let mut stmt = conn.prepare(&format!(
        "INSERT INTO user_message_part ({USER_MESSAGE_PART_COLUMNS})
         VALUES (
             :id, :user_message_id, :session_id, :position, :part_type,
             :text, :file_name, :file_url, :agent_name, :subtask_prompt, :subtask_description,
             :created_at, :updated_at
         )
         RETURNING *"
    ))?;
    let rows = from_rows::<UserMessagePart>(stmt.query(params.to_slice().as_slice())?);
    expect_one_returned_row("create_user_message_part", rows)
}

pub fn update(conn: &Connection, part: &UserMessagePart) -> Result<UserMessagePart, DatabaseError> {
    let mut updated = part.clone();
    updated.updated_at = chrono::Utc::now().naive_utc();

    let params = to_params_named_with_fields(
        &updated,
        &[
            "id",
            "user_message_id",
            "session_id",
            "position",
            "part_type",
            "text",
            "file_name",
            "file_url",
            "agent_name",
            "subtask_prompt",
            "subtask_description",
            "updated_at",
        ],
    )?;
    let mut stmt = conn.prepare(
        "UPDATE user_message_part
         SET
            user_message_id = :user_message_id,
            session_id = :session_id,
            position = :position,
            part_type = :part_type,
            text = :text,
            file_name = :file_name,
            file_url = :file_url,
            agent_name = :agent_name,
            subtask_prompt = :subtask_prompt,
            subtask_description = :subtask_description,
            updated_at = :updated_at
         WHERE id = :id
         RETURNING *",
    )?;
    let rows = from_rows::<UserMessagePart>(stmt.query(params.to_slice().as_slice())?);
    expect_one_returned_row("update_user_message_part", rows)
}

pub fn delete(conn: &Connection, part_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM user_message_part WHERE id = :id",
        named_params! {":id": part_id.to_string()},
    )?;
    assert_one_row_affected("delete_user_message_part", rows)
}
