use chrono::Utc;
use tokio_rusqlite::rusqlite::{Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::{assert_one_row_affected, check_returning_row_error};
use crate::backend::db::DatabaseError;
use crate::backend::Project;

pub fn row_to_project(row: &Row) -> Result<Project, tokio_rusqlite::rusqlite::Error> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        dir: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, dir, created_at, updated_at FROM projects ORDER BY updated_at DESC",
    )?;
    let projects = stmt
        .query_map([], row_to_project)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn get_project(conn: &Connection, project_id: Uuid) -> Result<Option<Project>, DatabaseError> {
    let mut stmt =
        conn.prepare("SELECT id, name, dir, created_at, updated_at FROM projects WHERE id = ?1")?;
    let project = stmt.query_row([project_id], row_to_project).optional()?;
    Ok(project)
}

pub fn create_project(conn: &Connection, project: &Project) -> Result<Project, DatabaseError> {
    let created = conn.query_row(
        "INSERT INTO projects (id, name, dir, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         RETURNING id, name, dir, created_at, updated_at",
        (
            &project.id,
            &project.name,
            &project.dir,
            &project.created_at,
            &project.updated_at,
        ),
        row_to_project,
    )?;
    Ok(created)
}

pub fn update_project(conn: &Connection, project: &Project) -> Result<Project, DatabaseError> {
    let updated = conn
        .query_row(
            "UPDATE projects
             SET name = ?2, dir = ?3, updated_at = ?4
             WHERE id = ?1
             RETURNING id, name, dir, created_at, updated_at",
            (
                &project.id,
                &project.name,
                &project.dir,
                Utc::now().naive_utc(),
            ),
            row_to_project,
        )
        .map_err(|e| check_returning_row_error("update_project", e))?;
    Ok(updated)
}

pub fn delete_project(conn: &Connection, project_id: Uuid) -> Result<(), DatabaseError> {
    let rows = conn.execute("DELETE FROM projects WHERE id = ?1", [project_id])?;
    assert_one_row_affected("delete_project", rows)
}
