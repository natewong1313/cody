use tokio_rusqlite::rusqlite::{self, params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::{db::DatabaseError, repo::user_message::UserMessage};

pub const USER_MESSAGE_COLUMNS: &str = "
id, session_id, agent, model_provider_id, model_id, system_prompt,
structured_output_type, tools_list, thinking_variant, created_at, updated_at
";

pub const USER_MESSAGE_COLUMN_COUNT: usize = 11;

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

pub fn list_messages_by_session(
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
         WHERE session_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2"
    ))?;

    let mut rows = stmt
        .query_map(params![session_id, i64::from(limit)], row_to_user_message)?
        .collect::<Result<Vec<_>, _>>()?;
    rows.reverse();
    Ok(rows)
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
