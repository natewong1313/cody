use serde_rusqlite::{from_rows, to_params, to_params_named, to_params_named_with_fields};
use tokio_rusqlite::named_params;
use tokio_rusqlite::rusqlite::Connection;
use uuid::Uuid;

use super::{assert_one_row_affected, expect_one_returned_row, now_utc_string};
use crate::backend::Project;
use crate::backend::db::DatabaseError;

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT * FROM projects ORDER BY updated_at DESC")?;
    let rows = from_rows::<Project>(stmt.query([])?);
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_project(conn: &Connection, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE id = :id")?;
    let mut rows = from_rows::<Project>(stmt.query(named_params! {":id": project_id.to_string()})?);
    Ok(rows.next().transpose()?)
}

pub fn create_project(conn: &Connection, project: &Project) -> Result<Project, DatabaseError> {
    let params = to_params_named(project)?;
    let mut stmt = conn.prepare(
        "
        INSERT INTO projects (id, name, dir, created_at, updated_at)
        VALUES (:id, :name, :dir, :created_at, :updated_at)
        RETURNING *
    ",
    )?;
    let rows = from_rows::<Project>(stmt.query(params.to_slice().as_slice())?);
    expect_one_returned_row("create_project", rows)
}

pub fn update_project(conn: &Connection, project: &Project) -> Result<Project, DatabaseError> {
    let params = to_params_named_with_fields(project, &["id", "name", "dir", "updated_at"])?;
    let mut stmt = conn.prepare(
        "
        UPDATE projects
        SET name = :name, dir = :dir, updated_at = :updated_at
        WHERE id = :id
        RETURNING *
    ",
    )?;
    let rows = from_rows::<Project>(stmt.query(params.to_slice().as_slice())?);
    expect_one_returned_row("update_project", rows)
}

pub fn delete_project(conn: &Connection, project_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute(
        "DELETE FROM projects WHERE id = :id",
        named_params! {":id": project_id.to_string()},
    )?;
    assert_one_row_affected("delete_project", rows)
}
