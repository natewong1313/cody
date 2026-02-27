use tokio_rusqlite::rusqlite::{Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error, now_utc_string};
use crate::backend::db::DatabaseError;
use crate::backend::Session;

const SELECT_SESSION_COLUMNS: &str = "
id, project_id, parent_session_id, show_in_gui, name, harness_type, harness_session_id,
dir, summary_additions, summary_deletions, summary_files, created_at, updated_at
";

pub fn row_to_session(row: &Row) -> Result<Session, tokio_rusqlite::rusqlite::Error> {
    Ok(Session {
        id: row.get(0)?,
        project_id: row.get(1)?,
        parent_session_id: row.get(2)?,
        show_in_gui: row.get(3)?,
        name: row.get(4)?,
        harness_type: row.get(5)?,
        harness_session_id: row.get(6)?,
        dir: row.get(7)?,
        summary_additions: row.get(8)?,
        summary_deletions: row.get(9)?,
        summary_files: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

pub fn list_sessions_by_project(
    conn: &Connection,
    project_id: Uuid,
) -> Result<Vec<Session>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_SESSION_COLUMNS}
         FROM sessions
         WHERE project_id = ?1
         ORDER BY updated_at DESC"
    ))?;
    let sessions = stmt
        .query_map([project_id], row_to_session)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sessions)
}

pub fn get_session(conn: &Connection, session_id: Uuid) -> Result<Option<Session>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_SESSION_COLUMNS}
         FROM sessions
         WHERE id = ?1"
    ))?;
    let session = stmt.query_row([session_id], row_to_session).optional()?;
    Ok(session)
}

pub fn create_session(conn: &Connection, session: &Session) -> Result<Session, DatabaseError> {
    let created = conn.query_row(
        &format!(
            "INSERT INTO sessions (
                id, project_id, parent_session_id, show_in_gui, name, harness_type,
                harness_session_id, dir, summary_additions, summary_deletions, summary_files,
                created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            RETURNING {SELECT_SESSION_COLUMNS}"
        ),
        (
            &session.id,
            &session.project_id,
            &session.parent_session_id,
            &session.show_in_gui,
            &session.name,
            &session.harness_type,
            &session.harness_session_id,
            &session.dir,
            &session.summary_additions,
            &session.summary_deletions,
            &session.summary_files,
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
            &format!(
                "UPDATE sessions
                 SET
                    project_id = ?2,
                    parent_session_id = ?3,
                    show_in_gui = ?4,
                    name = ?5,
                    harness_type = ?6,
                    harness_session_id = ?7,
                    dir = ?8,
                    summary_additions = ?9,
                    summary_deletions = ?10,
                    summary_files = ?11,
                    updated_at = ?12
                 WHERE id = ?1
                 RETURNING {SELECT_SESSION_COLUMNS}"
            ),
            (
                &session.id,
                &session.project_id,
                &session.parent_session_id,
                &session.show_in_gui,
                &session.name,
                &session.harness_type,
                &session.harness_session_id,
                &session.dir,
                &session.summary_additions,
                &session.summary_deletions,
                &session.summary_files,
                now_utc_string(),
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
