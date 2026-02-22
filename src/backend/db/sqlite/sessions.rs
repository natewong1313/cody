use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use super::{assert_one_row_affected, check_returning_row_error, row_to_session};
use crate::backend::Session;
use crate::backend::db::DatabaseError;

pub(super) fn list_sessions_by_project(
    conn: &Connection,
    project_id: uuid::Uuid,
) -> Result<Vec<Session>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, show_in_gui, name, created_at, updated_at
         FROM sessions
         WHERE project_id = ?1
         ORDER BY updated_at DESC",
    )?;
    let sessions = stmt
        .query_map([project_id], row_to_session)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sessions)
}

pub(super) fn get_session(
    conn: &Connection,
    session_id: uuid::Uuid,
) -> Result<Option<Session>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, show_in_gui, name, created_at, updated_at
         FROM sessions
         WHERE id = ?1",
    )?;
    let session = stmt.query_row([session_id], row_to_session).optional()?;
    Ok(session)
}

pub(super) fn create_session(
    conn: &Connection,
    session: &Session,
) -> Result<Session, DatabaseError> {
    let created = conn.query_row(
        "INSERT INTO sessions (id, project_id, show_in_gui, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING id, project_id, show_in_gui, name, created_at, updated_at",
        (
            &session.id,
            &session.project_id,
            &session.show_in_gui,
            &session.name,
            &session.created_at,
            &session.updated_at,
        ),
        row_to_session,
    )?;
    Ok(created)
}

pub(super) fn update_session(
    conn: &Connection,
    session: &Session,
) -> Result<Session, DatabaseError> {
    let updated = conn
        .query_row(
            "UPDATE sessions
             SET project_id = ?2, show_in_gui = ?3, name = ?4, updated_at = ?5
             WHERE id = ?1
             RETURNING id, project_id, show_in_gui, name, created_at, updated_at",
            (
                &session.id,
                &session.project_id,
                &session.show_in_gui,
                &session.name,
                Utc::now().naive_utc(),
            ),
            row_to_session,
        )
        .map_err(|e| check_returning_row_error("update_session", e))?;
    Ok(updated)
}

pub(super) fn delete_session(
    conn: &Connection,
    session_id: uuid::Uuid,
) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
    assert_one_row_affected("delete_session", rows)
}
