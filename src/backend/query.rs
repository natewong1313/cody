use super::Project;
use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

pub fn query_all_projects(db_conn: &Connection) -> anyhow::Result<Vec<Project>> {
    let mut stmt = db_conn.prepare("SELECT id, name, dir FROM projects")?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                dir: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn query_project_by_id(db_conn: &Connection, id: &Uuid) -> anyhow::Result<Option<Project>> {
    let mut stmt = db_conn.prepare("SELECT id, name, dir FROM projects WHERE id = ?1")?;
    let project = stmt
        .query_row([id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                dir: row.get(2)?,
            })
        })
        .optional()?;
    Ok(project)
}
