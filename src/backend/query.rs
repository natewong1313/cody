use super::{Project, Session};
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

pub fn query_all_sessions_by_project_id(
    db_conn: &Connection,
    project_id: &Uuid,
) -> anyhow::Result<Vec<Session>> {
    let mut stmt =
        db_conn.prepare("SELECT id, project_id, name FROM sessions WHERE project_id = ?1")?;
    let sessions = stmt
        .query_map([project_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sessions)
}

pub fn query_session_by_id(db_conn: &Connection, id: &Uuid) -> anyhow::Result<Option<Session>> {
    let mut stmt = db_conn.prepare("SELECT id, project_id, name FROM sessions WHERE id = ?1")?;
    let session = stmt
        .query_row([id], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .optional()?;
    Ok(session)
}
