use rusqlite_migration::{M, Migrations};

const SQLITE_MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "CREATE TABLE projects (
            id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
            name TEXT NOT NULL CHECK(length(trim(name)) > 0),
            dir TEXT NOT NULL CHECK(length(trim(dir)) > 0),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    ),
    M::up(
        "CREATE TABLE sessions (
            id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
            project_id BLOB NOT NULL CHECK(length(project_id) = 16) REFERENCES projects(id) ON DELETE CASCADE,

            show_in_gui NOT NULL DEFAULT 0 CHECK(show_in_gui IN (0, 1)),

            name TEXT NOT NULL DEFAULT 'New Session',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    ),
];
pub const SQLITE_MIGRATIONS: Migrations<'_> = Migrations::from_slice(SQLITE_MIGRATIONS_SLICE);
