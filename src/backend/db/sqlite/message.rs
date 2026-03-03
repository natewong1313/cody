use tokio_rusqlite::rusqlite::{self, params, Connection, Row};
use uuid::Uuid;

use super::{assistant_message, user_message};
use crate::backend::{db::DatabaseError, repo::message::Message};

fn row_to_message(row: &Row) -> Result<Message, rusqlite::Error> {
    let kind: String = row.get(0)?;
    match kind.as_str() {
        "user" => Ok(Message::User(user_message::row_to_user_message_at(row, 1)?)),
        "assistant" => Ok(Message::Assistant(
            assistant_message::row_to_assistant_message_at(
                row,
                1 + user_message::USER_MESSAGE_COLUMN_COUNT,
            )?,
        )),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown message kind: {kind}").into(),
        )),
    }
}

fn qualify_columns(columns: &str, alias: &str) -> String {
    columns
        .split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(|c| format!("{alias}.{c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn list_messages_by_session(
    conn: &Connection,
    session_id: Uuid,
    limit: u32,
) -> Result<Vec<Message>, DatabaseError> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let user_columns = qualify_columns(user_message::USER_MESSAGE_COLUMNS, "u");
    let assistant_columns = qualify_columns(assistant_message::ASSISTANT_MESSAGE_COLUMNS, "a");

    let mut stmt = conn.prepare(&format!(
        "WITH latest AS (
            SELECT kind, id, created_at
            FROM (
                SELECT 'user' AS kind, id, created_at
                FROM user_message
                WHERE session_id = ?1

                UNION ALL

                SELECT 'assistant' AS kind, id, created_at
                FROM assistant_message
                WHERE session_id = ?1
            )
            ORDER BY created_at DESC
            LIMIT ?2
         )
         SELECT
            latest.kind,
            {user_columns},
            {assistant_columns}
         FROM latest
         LEFT JOIN user_message u ON latest.kind = 'user' AND u.id = latest.id
         LEFT JOIN assistant_message a ON latest.kind = 'assistant' AND a.id = latest.id
         ORDER BY latest.created_at DESC",
        user_columns = user_columns,
        assistant_columns = assistant_columns,
    ))?;

    let mut rows = stmt
        .query_map(params![session_id, i64::from(limit)], row_to_message)?
        .collect::<Result<Vec<_>, _>>()?;
    rows.reverse();
    Ok(rows)
}
