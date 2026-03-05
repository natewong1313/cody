use tokio_rusqlite::rusqlite::{self, params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::backend::{
    db::{
        sqlite::{assert_one_row_affected, check_returning_row_error, now_utc_string},
        DatabaseError,
    },
    repo::user_message_part::UserMessagePart,
};

const USER_MESSAGE_PART_COLUMNS: &str = "
id, user_message_id, session_id, position, part_type,
text, file_name, file_url, agent_name, subtask_prompt, subtask_description,
created_at, updated_at
";

pub fn row_to_user_message_part(row: &Row) -> Result<UserMessagePart, rusqlite::Error> {
    Ok(UserMessagePart {
        id: row.get(0)?,
        user_message_id: row.get(1)?,
        session_id: row.get(2)?,
        position: row.get(3)?,
        part_type: row.get(4)?,
        text: row.get(5)?,
        file_name: row.get(6)?,
        file_url: row.get(7)?,
        agent_name: row.get(8)?,
        subtask_prompt: row.get(9)?,
        subtask_description: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

pub fn get(conn: &Connection, part_id: Uuid) -> Result<Option<UserMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {USER_MESSAGE_PART_COLUMNS}
         FROM user_message_part
         WHERE id = ?1"
    ))?;
    Ok(stmt
        .query_row([part_id], row_to_user_message_part)
        .optional()?)
}

pub fn create(conn: &Connection, part: &UserMessagePart) -> Result<UserMessagePart, DatabaseError> {
    let created = conn.query_row(
        &format!(
            "INSERT INTO user_message_part ({USER_MESSAGE_PART_COLUMNS})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             RETURNING {USER_MESSAGE_PART_COLUMNS}"
        ),
        params![
            &part.id,
            &part.user_message_id,
            &part.session_id,
            &part.position,
            &part.part_type,
            &part.text,
            &part.file_name,
            &part.file_url,
            &part.agent_name,
            &part.subtask_prompt,
            &part.subtask_description,
            &part.created_at,
            &part.updated_at,
        ],
        row_to_user_message_part,
    )?;
    Ok(created)
}

pub fn update(conn: &Connection, part: &UserMessagePart) -> Result<UserMessagePart, DatabaseError> {
    let updated = conn
        .query_row(
            &format!(
                "UPDATE user_message_part
                 SET
                    user_message_id = ?2,
                    session_id = ?3,
                    position = ?4,
                    part_type = ?5,
                    text = ?6,
                    file_name = ?7,
                    file_url = ?8,
                    agent_name = ?9,
                    subtask_prompt = ?10,
                    subtask_description = ?11,
                    updated_at = ?12
                 WHERE id = ?1
                 RETURNING {USER_MESSAGE_PART_COLUMNS}"
            ),
            params![
                &part.id,
                &part.user_message_id,
                &part.session_id,
                &part.position,
                &part.part_type,
                &part.text,
                &part.file_name,
                &part.file_url,
                &part.agent_name,
                &part.subtask_prompt,
                &part.subtask_description,
                now_utc_string(),
            ],
            row_to_user_message_part,
        )
        .map_err(|e| check_returning_row_error("update_user_message_part", e))?;
    Ok(updated)
}

pub fn delete(conn: &Connection, part_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM user_message_part WHERE id = ?1", [part_id])?;
    assert_one_row_affected("delete_user_message_part", rows)
}
