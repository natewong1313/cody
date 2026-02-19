use chrono::{NaiveDateTime, Utc};
use rusqlite::{OptionalExtension, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    backend::{
        db::{assert_one_row_affected, check_returning_row_error, DatabaseError},
        BackendContext,
    },
    with_db_conn,
};

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub dir: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Error)]
pub enum ProjectRepoError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
}

impl From<rusqlite::Error> for ProjectRepoError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(DatabaseError::from(err))
    }
}

impl Project {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            dir: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

pub struct ProjectRepo {
    ctx: BackendContext,
}

impl ProjectRepo {
    pub fn new(ctx: BackendContext) -> Self {
        Self { ctx }
    }

    pub fn list(&self) -> Result<Vec<Project>, ProjectRepoError> {
        let projects = with_db_conn!(self, conn, {
            let mut stmt = conn.prepare(
                "SELECT id, name, dir, created_at, updated_at FROM projects ORDER BY updated_at DESC",
            )?;
            let projects = stmt
                .query_map([], Project::from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok::<Vec<Project>, DatabaseError>(projects)
        })
        .map_err(ProjectRepoError::from)?;

        Ok(projects)
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Project>, ProjectRepoError> {
        let project = with_db_conn!(self, conn, {
            let mut stmt = conn.prepare(
                "SELECT id, name, dir, created_at, updated_at FROM projects WHERE id = ?1",
            )?;
            let project = stmt.query_row([id], Project::from_row).optional()?;
            Ok::<Option<Project>, DatabaseError>(project)
        })
        .map_err(ProjectRepoError::from)?;

        Ok(project)
    }

    pub fn create(&self, project: &Project) -> Result<Project, ProjectRepoError> {
        let created = with_db_conn!(self, conn, {
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
                Project::from_row,
            )?;
            Ok::<Project, DatabaseError>(created)
        })
        .map_err(ProjectRepoError::from)?;

        Ok(created)
    }

    pub fn update(&self, project: &Project) -> Result<Project, ProjectRepoError> {
        let updated = with_db_conn!(self, conn, {
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
                    Project::from_row,
                )
                .map_err(|e| check_returning_row_error("update_project", e))?;
            Ok::<Project, DatabaseError>(updated)
        })
        .map_err(ProjectRepoError::from)?;

        Ok(updated)
    }

    pub fn delete(&self, project_id: &Uuid) -> Result<(), ProjectRepoError> {
        with_db_conn!(self, conn, {
            let rows = conn.execute("DELETE FROM projects WHERE id = ?1", [project_id])?;
            assert_one_row_affected("delete_project", rows)
        })
        .map_err(ProjectRepoError::from)?;

        Ok(())
    }
}
