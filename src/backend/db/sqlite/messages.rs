use chrono::Utc;
use tokio_rusqlite::Row;
use tokio_rusqlite::rusqlite::{self, Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::backend::db::DatabaseError;
use crate::backend::repo::message::{Message, MessagePart};

type MessageWithPartRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn map_message_with_part_row(row: &Row<'_>) -> rusqlite::Result<MessageWithPartRow> {
    Ok((
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
        row.get::<_, String>(5)?,
        row.get::<_, String>(6)?,
        row.get::<_, String>(7)?,
        row.get::<_, Option<String>>(8)?,
        row.get::<_, Option<String>>(9)?,
        row.get::<_, Option<String>>(10)?,
        row.get::<_, Option<String>>(11)?,
        row.get::<_, Option<String>>(12)?,
    ))
}

fn message_part_from_row(
    part_id: Option<String>,
    part_message_id: Option<String>,
    part_type: Option<String>,
    part_text: Option<String>,
    part_tool_json: Option<String>,
) -> Option<MessagePart> {
    match (
        part_id,
        part_message_id,
        part_type,
        part_text,
        part_tool_json,
    ) {
        (Some(id), Some(message_id), Some(part_type), Some(text), Some(tool_json)) => {
            Some(MessagePart {
                id,
                message_id,
                part_type,
                text,
                tool_json,
            })
        }
        _ => None,
    }
}

fn fetch_session_message_rows(
    conn: &Connection,
    session_id: Uuid,
    limit: Option<i32>,
) -> Result<Vec<MessageWithPartRow>, DatabaseError> {
    let query_with_limit = "WITH selected_messages AS (
            SELECT id, session_id, role, created_at, completed_at, parent_id, provider_id, model_id, error_json
            FROM session_messages
            WHERE session_id = ?1 AND removed_at IS NULL
            ORDER BY created_at ASC, id ASC
            LIMIT ?2
        )
        SELECT
            m.id,
            m.role,
            m.created_at,
            m.completed_at,
            m.parent_id,
            m.provider_id,
            m.model_id,
            m.error_json,
            p.id,
            p.message_id,
            p.part_type,
            p.text,
            p.tool_json
        FROM selected_messages m
        LEFT JOIN session_message_parts p
            ON p.session_id = m.session_id
           AND p.message_id = m.id
        ORDER BY m.created_at ASC, m.id ASC, p.id ASC";

    let query_without_limit = "SELECT
            m.id,
            m.role,
            m.created_at,
            m.completed_at,
            m.parent_id,
            m.provider_id,
            m.model_id,
            m.error_json,
            p.id,
            p.message_id,
            p.part_type,
            p.text,
            p.tool_json
        FROM session_messages m
        LEFT JOIN session_message_parts p
            ON p.session_id = m.session_id
           AND p.message_id = m.id
        WHERE m.session_id = ?1 AND m.removed_at IS NULL
        ORDER BY m.created_at ASC, m.id ASC, p.id ASC";

    let mut stmt = conn.prepare(if limit.is_some() {
        query_with_limit
    } else {
        query_without_limit
    })?;

    if let Some(limit) = limit {
        stmt.query_map(params![session_id, limit], map_message_with_part_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)
    } else {
        stmt.query_map(params![session_id], map_message_with_part_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)
    }
}

pub fn get_session_id_by_harness_id(
    conn: &Connection,
    harness_id: &str,
) -> Result<Option<Uuid>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT id FROM sessions WHERE harness_id = ?1")?;
    let session_id = stmt.query_row([harness_id], |row| row.get(0)).optional()?;
    Ok(session_id)
}

pub fn upsert_session_message(conn: &Connection, message: &Message) -> Result<(), DatabaseError> {
    let updated_at = Utc::now().naive_utc().to_string();
    conn.execute(
        "INSERT INTO session_messages (
            session_id, id, role, created_at, completed_at, parent_id,
            provider_id, model_id, error_json, removed_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10)
        ON CONFLICT(session_id, id) DO UPDATE SET
            role = excluded.role,
            created_at = excluded.created_at,
            completed_at = excluded.completed_at,
            parent_id = excluded.parent_id,
            provider_id = excluded.provider_id,
            model_id = excluded.model_id,
            error_json = excluded.error_json,
            removed_at = NULL,
            updated_at = excluded.updated_at",
        params![
            message.session_id,
            message.id,
            message.role,
            message.created_at,
            message.completed_at,
            message.parent_id,
            message.provider_id,
            message.model_id,
            message.error_json,
            updated_at,
        ],
    )?;

    Ok(())
}

pub fn upsert_session_message_with_parts(
    conn: &Connection,
    message: &Message,
) -> Result<(), DatabaseError> {
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION;")?;

    if let Err(err) = (|| -> Result<(), DatabaseError> {
        upsert_session_message(conn, message)?;
        for part in &message.parts {
            upsert_session_message_part(conn, message.session_id, part, None)?;
        }
        Ok(())
    })() {
        let _ = conn.execute_batch("ROLLBACK;");
        return Err(err);
    }

    conn.execute_batch("COMMIT;")?;
    Ok(())
}

pub fn ensure_session_message_exists(
    conn: &Connection,
    session_id: Uuid,
    message_id: &str,
) -> Result<(), DatabaseError> {
    let updated_at = Utc::now().naive_utc().to_string();
    conn.execute(
        "INSERT INTO session_messages (
            session_id, id, role, created_at, completed_at, parent_id,
            provider_id, model_id, error_json, removed_at, updated_at
        ) VALUES (?1, ?2, 'assistant', ?3, '', '', '', '', '', NULL, ?3)
        ON CONFLICT(session_id, id) DO NOTHING",
        params![session_id, message_id, updated_at],
    )?;

    Ok(())
}

pub fn mark_session_message_removed(
    conn: &Connection,
    session_id: Uuid,
    message_id: &str,
) -> Result<(), DatabaseError> {
    let updated_at = Utc::now().naive_utc().to_string();
    conn.execute(
        "UPDATE session_messages
         SET removed_at = ?3, updated_at = ?3
         WHERE session_id = ?1 AND id = ?2",
        params![session_id, message_id, updated_at],
    )?;

    Ok(())
}

pub fn upsert_session_message_part(
    conn: &Connection,
    session_id: Uuid,
    part: &MessagePart,
    delta: Option<&str>,
) -> Result<(), DatabaseError> {
    let updated_at = Utc::now().naive_utc().to_string();
    conn.execute(
        "INSERT INTO session_message_parts (
            session_id, message_id, id, part_type, text, tool_json, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(session_id, message_id, id) DO UPDATE SET
            part_type = excluded.part_type,
            text = CASE
                WHEN excluded.text <> '' THEN excluded.text
                WHEN ?8 IS NOT NULL
                     AND ?8 <> ''
                     AND substr(session_message_parts.text, -length(?8)) <> ?8
                    THEN session_message_parts.text || ?8
                ELSE session_message_parts.text
            END,
            tool_json = excluded.tool_json,
            updated_at = excluded.updated_at",
        params![
            session_id,
            part.message_id,
            part.id,
            part.part_type,
            part.text,
            part.tool_json,
            updated_at,
            delta,
        ],
    )?;

    Ok(())
}

pub fn list_session_messages(
    conn: &Connection,
    session_id: Uuid,
    limit: Option<i32>,
) -> Result<Vec<Message>, DatabaseError> {
    let rows = fetch_session_message_rows(conn, session_id, limit)?;

    let mut messages: Vec<Message> = Vec::new();
    for (
        id,
        role,
        created_at,
        completed_at,
        parent_id,
        provider_id,
        model_id,
        error_json,
        part_id,
        part_message_id,
        part_type,
        part_text,
        part_tool_json,
    ) in rows
    {
        let needs_new_message = messages.last().is_none_or(|existing| existing.id != id);
        if needs_new_message {
            messages.push(Message {
                id: id.clone(),
                session_id,
                role,
                created_at,
                completed_at,
                parent_id,
                provider_id,
                model_id,
                error_json,
                parts: Vec::new(),
            });
        }

        if let Some(part) = message_part_from_row(
            part_id,
            part_message_id,
            part_type,
            part_text,
            part_tool_json,
        )
            && let Some(message) = messages.last_mut()
        {
            message.parts.push(part);
        }
    }

    Ok(messages)
}
