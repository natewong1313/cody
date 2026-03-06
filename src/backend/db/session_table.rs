use serde_rusqlite::{from_rows, to_params_named, to_params_named_with_fields};
use tokio_rusqlite::named_params;
use tokio_rusqlite::rusqlite::Connection;
use uuid::Uuid;

use crate::backend::db::DatabaseError;
use crate::backend::models::session_model::SessionModel;

const SESSION_COLUMNS: &str = "\nid, project_id, parent_session_id, show_in_gui, name, harness_type, harness_session_id,\ndir, summary_additions, summary_deletions, summary_files, created_at, updated_at\n";

pub fn list_by_project(
    conn: &Connection,
    project_id: Uuid,
) -> Result<Vec<SessionModel>, DatabaseError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SESSION_COLUMNS}
         FROM sessions
         WHERE project_id = :project_id
         ORDER BY updated_at DESC"
    ))?;
    let rows = from_rows::<SessionModel>(
        stmt.query(named_params! {":project_id": project_id.to_string()})?,
    );
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get(conn: &Connection, session_id: Uuid) -> Result<Option<SessionModel>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT * FROM sessions WHERE id = :id")?;
    let mut rows =
        from_rows::<SessionModel>(stmt.query(named_params! {":id": session_id.to_string()})?);
    Ok(rows.next().transpose()?)
}

pub fn create(conn: &Connection, session: &SessionModel) -> Result<SessionModel, DatabaseError> {
    let params = to_params_named(session)?;
    let mut stmt = conn.prepare(
        &format!("
        INSERT INTO sessions ({SESSION_COLUMNS})
        VALUES (
            :id, :project_id, :parent_session_id, :show_in_gui, :name, :harness_type, :harness_session_id,
            :dir, :summary_additions, :summary_deletions, :summary_files, :created_at, :updated_at
        )
        RETURNING *
    "),
    )?;
    let rows = from_rows::<SessionModel>(stmt.query(params.to_slice().as_slice())?);
    super::expect_one_returned_row("create_session", rows)
}

pub fn update(conn: &Connection, session: &SessionModel) -> Result<SessionModel, DatabaseError> {
    let mut updated = session.clone();
    updated.updated_at = chrono::Utc::now().naive_utc();

    let params = to_params_named_with_fields(
        &updated,
        &[
            "id",
            "project_id",
            "parent_session_id",
            "show_in_gui",
            "name",
            "harness_type",
            "harness_session_id",
            "dir",
            "summary_additions",
            "summary_deletions",
            "summary_files",
            "updated_at",
        ],
    )?;
    let mut stmt = conn.prepare(
        "
        UPDATE sessions
        SET
            project_id = :project_id,
            parent_session_id = :parent_session_id,
            show_in_gui = :show_in_gui,
            name = :name,
            harness_type = :harness_type,
            harness_session_id = :harness_session_id,
            dir = :dir,
            summary_additions = :summary_additions,
            summary_deletions = :summary_deletions,
            summary_files = :summary_files,
            updated_at = :updated_at
        WHERE id = :id
        RETURNING *
    ",
    )?;
    let rows = from_rows::<SessionModel>(stmt.query(params.to_slice().as_slice())?);
    super::expect_one_returned_row("update_session", rows)
}

pub fn delete(conn: &Connection, session_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM sessions WHERE id = :id",
        named_params! {":id": session_id.to_string()},
    )?;
    super::assert_one_row_affected("delete_session", rows)
}
