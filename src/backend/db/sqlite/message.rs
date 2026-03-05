use serde_rusqlite::from_row;
use tokio_rusqlite::rusqlite::{self, Connection, Row, params};
use uuid::Uuid;

use crate::backend::{
    db::DatabaseError,
    repo::{assistant_message::AssistantMessage, message::Message, user_message::UserMessage},
};

fn row_to_message(row: &Row) -> Result<Message, DatabaseError> {
    let kind: String = row.get(0)?;

    match kind.as_str() {
        "user" => from_row::<UserMessage>(row)
            .map(Message::User)
            .map_err(Into::into),
        "assistant" => from_row::<AssistantMessage>(row)
            .map(Message::Assistant)
            .map_err(Into::into),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown message kind: {kind}").into(),
        )
        .into()),
    }
}

pub fn list_messages_by_session(
    conn: &Connection,
    session_id: Uuid,
    limit: u32,
) -> Result<Vec<Message>, DatabaseError> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
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
            COALESCE(u.id, a.id) AS id,
            a.harness_message_id AS harness_message_id,
            COALESCE(u.session_id, a.session_id) AS session_id,
            a.user_message_id AS user_message_id,
            COALESCE(u.agent, a.agent) AS agent,
            COALESCE(u.model_provider_id, a.model_provider_id) AS model_provider_id,
            COALESCE(u.model_id, a.model_id) AS model_id,
            u.system_prompt AS system_prompt,
            u.structured_output_type AS structured_output_type,
            u.tools_list AS tools_list,
            u.thinking_variant AS thinking_variant,
            a.cwd AS cwd,
            a.root AS root,
            a.cost AS cost,
            a.token_total AS token_total,
            a.token_input AS token_input,
            a.token_output AS token_output,
            a.token_reasoning AS token_reasoning,
            a.token_cache_read AS token_cache_read,
            a.token_cache_write AS token_cache_write,
            a.error_message AS error_message,
            COALESCE(u.created_at, a.created_at) AS created_at,
            COALESCE(u.updated_at, a.updated_at) AS updated_at,
            a.completed_at AS completed_at
         FROM latest
         LEFT JOIN user_message u ON latest.kind = 'user' AND u.id = latest.id
         LEFT JOIN assistant_message a ON latest.kind = 'assistant' AND a.id = latest.id
         ORDER BY latest.created_at DESC",
    )?;

    let mut rows = stmt
        .query_and_then(params![session_id, i64::from(limit)], row_to_message)?
        .collect::<Result<Vec<_>, DatabaseError>>()?;
    rows.reverse();
    Ok(rows)
}
