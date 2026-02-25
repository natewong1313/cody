use chrono::Utc;
use tokio_rusqlite::rusqlite::{Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error};
use crate::backend::Session;
use crate::backend::db::DatabaseError;

pub fn row_to_session(row: &Row) -> Result<Session, tokio_rusqlite::rusqlite::Error> {
    Ok(Session {
        id: row.get(0)?,
        project_id: row.get(1)?,
        show_in_gui: row.get(2)?,
        name: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn list_sessions_by_project(
    conn: &Connection,
    project_id: Uuid,
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

pub fn get_session(conn: &Connection, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, show_in_gui, name, created_at, updated_at
         FROM sessions
         WHERE id = ?1",
    )?;
    let session = stmt.query_row([session_id], row_to_session).optional()?;
    Ok(session)
}

pub fn create_session(conn: &Connection, session: &Session) -> Result<Session, DatabaseError> {
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

pub fn update_session(conn: &Connection, session: &Session) -> Result<Session, DatabaseError> {
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

pub fn delete_session(conn: &Connection, session_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
    assert_one_row_affected("delete_session", rows)
}

pub fn set_session_harness_id(
    conn: &Connection,
    session_id: Uuid,
    harness_id: &str,
) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "UPDATE sessions SET harness_id = ?2 WHERE id = ?1",
        (session_id, harness_id),
    )?;
    assert_one_row_affected("set_session_harness_id", rows)
}

pub fn get_session_harness_id(
    conn: &Connection,
    session_id: Uuid,
) -> Result<Option<String>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT harness_id FROM sessions WHERE id = ?1")?;
    let harness_id = stmt.query_row([session_id], |row| row.get(0)).optional()?;
    Ok(harness_id)
}
