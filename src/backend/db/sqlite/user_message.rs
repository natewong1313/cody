use tokio_rusqlite::rusqlite::{self, Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{
    db::DatabaseError,
    repo::user_message::{UserMessage, UserMessagePart},
};

pub const USER_MESSAGE_COLUMNS: &str = "
id, session_id, agent, model_provider_id, model_id, system_prompt,
structured_output_type, tools_list, thinking_variant, created_at, updated_at
";

pub const USER_MESSAGE_COLUMN_COUNT: usize = 11;

const USER_MESSAGE_PART_COLUMNS: &str = "
id, user_message_id, session_id, position, part_type,
text, file_name, file_url, agent_name, subtask_prompt, subtask_description,
created_at, updated_at
";

pub fn row_to_user_message(row: &Row) -> Result<UserMessage, rusqlite::Error> {
    row_to_user_message_at(row, 0)
}

pub fn row_to_user_message_at(row: &Row, start: usize) -> Result<UserMessage, rusqlite::Error> {
    Ok(UserMessage {
        id: row.get(start)?,
        session_id: row.get(start + 1)?,
        agent: row.get(start + 2)?,
        model_provider_id: row.get(start + 3)?,
        model_id: row.get(start + 4)?,
        system_prompt: row.get(start + 5)?,
        structured_output_type: row.get(start + 6)?,
        tools_list: row.get(start + 7)?,
        thinking_variant: row.get(start + 8)?,
        created_at: row.get(start + 9)?,
        updated_at: row.get(start + 10)?,
    })
}

pub fn get_user_message(
    conn: &Connection,
    user_message_id: Uuid,
) -> Result<Option<UserMessage>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {USER_MESSAGE_COLUMNS}
         FROM user_message
         WHERE id = ?1"
    ))?;
    Ok(stmt
        .query_row([user_message_id], row_to_user_message)
        .optional()?)
}

pub fn create_user_message(
    conn: &Connection,
    user_message: &UserMessage,
) -> Result<UserMessage, DatabaseError> {
    let rows = conn.execute(
        &format!(
            "INSERT INTO user_message ({USER_MESSAGE_COLUMNS})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        ),
        params![
            &user_message.id,
            &user_message.session_id,
            &user_message.agent,
            &user_message.model_provider_id,
            &user_message.model_id,
            &user_message.system_prompt,
            &user_message.structured_output_type,
            &user_message.tools_list,
            &user_message.thinking_variant,
            &user_message.created_at,
            &user_message.updated_at,
        ],
    )?;
    assert_one_row_affected("create_user_message", rows)?;
    get_user_message(conn, user_message.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "create_user_message",
        expected: 1,
        actual: 0,
    })
}

pub fn update_user_message(
    conn: &Connection,
    user_message: &UserMessage,
) -> Result<UserMessage, DatabaseError> {
    let rows = conn
        .execute(
            "UPDATE user_message
             SET
                session_id = ?2,
                agent = ?3,
                model_provider_id = ?4,
                model_id = ?5,
                system_prompt = ?6,
                structured_output_type = ?7,
                tools_list = ?8,
                thinking_variant = ?9,
                updated_at = ?10
             WHERE id = ?1",
            params![
                &user_message.id,
                &user_message.session_id,
                &user_message.agent,
                &user_message.model_provider_id,
                &user_message.model_id,
                &user_message.system_prompt,
                &user_message.structured_output_type,
                &user_message.tools_list,
                &user_message.thinking_variant,
                now_utc_string(),
            ],
        )
        .map_err(|e| check_returning_row_error("update_user_message", e))?;
    assert_one_row_affected("update_user_message", rows)?;
    get_user_message(conn, user_message.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "update_user_message",
        expected: 1,
        actual: 0,
    })
}

pub fn delete_user_message(conn: &Connection, user_message_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM user_message WHERE id = ?1", [user_message_id])?;
    assert_one_row_affected("delete_user_message", rows)
}

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

pub fn get_user_message_part(
    conn: &Connection,
    part_id: Uuid,
) -> Result<Option<UserMessagePart>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {USER_MESSAGE_PART_COLUMNS}
         FROM user_message_part
         WHERE id = ?1"
    ))?;
    Ok(stmt
        .query_row([part_id], row_to_user_message_part)
        .optional()?)
}

pub fn create_user_message_part(
    conn: &Connection,
    part: &UserMessagePart,
) -> Result<UserMessagePart, DatabaseError> {
    let rows = conn.execute(
        &format!(
            "INSERT INTO user_message_part ({USER_MESSAGE_PART_COLUMNS})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
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
    )?;
    assert_one_row_affected("create_user_message_part", rows)?;
    get_user_message_part(conn, part.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "create_user_message_part",
        expected: 1,
        actual: 0,
    })
}

pub fn update_user_message_part(
    conn: &Connection,
    part: &UserMessagePart,
) -> Result<UserMessagePart, DatabaseError> {
    let rows = conn
        .execute(
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
             WHERE id = ?1",
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
        )
        .map_err(|e| check_returning_row_error("update_user_message_part", e))?;
    assert_one_row_affected("update_user_message_part", rows)?;
    get_user_message_part(conn, part.id)?.ok_or(DatabaseError::UnexpectedRowsAffected {
        op: "update_user_message_part",
        expected: 1,
        actual: 0,
    })
}

pub fn delete_user_message_part(conn: &Connection, part_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM user_message_part WHERE id = ?1", [part_id])?;
    assert_one_row_affected("delete_user_message_part", rows)
}
