use migrations::MIGRATIONS;
use rusqlite::Connection;
use thiserror::Error;
mod migrations;

#[derive(Error, Debug)]
pub enum DatabaseStartupError {
    #[error("Error establishing connection {0}")]
    Connection(#[from] rusqlite::Error),
    #[error("Error migrating db {0}")]
    Migration(#[from] rusqlite_migration::Error),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Generic database error {0}")]
    QueryError(#[from] rusqlite::Error),
    #[error("Db conn lock poisoned")]
    PoisonedLock,
    #[error("{op} unexpected rows affected, expected {expected} got {actual}")]
    UnexpectedRowsAffected {
        op: &'static str,
        expected: usize,
        actual: usize,
    },
}

pub fn new_db_connection() -> Result<Connection, DatabaseStartupError> {
    let mut conn = Connection::open("./cody.db")?;
    conn.pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    MIGRATIONS.to_latest(&mut conn)?;
    Ok(conn)
}

/// Helper function to make sure updates are updating
pub fn check_returning_row_error(op: &'static str, err: rusqlite::Error) -> DatabaseError {
    match err {
        rusqlite::Error::QueryReturnedNoRows => DatabaseError::UnexpectedRowsAffected {
            op,
            expected: 1,
            actual: 0,
        },
        other => DatabaseError::QueryError(other),
    }
}

/// Helper function to make sure rows are actually being deleted
pub fn assert_one_row_affected(
    op: &'static str,
    rows_affected: usize,
) -> Result<(), DatabaseError> {
    if rows_affected == 1 {
        Ok(())
    } else {
        Err(DatabaseError::UnexpectedRowsAffected {
            op,
            expected: 1,
            actual: rows_affected,
        })
    }
}
