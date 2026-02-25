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
    M::up(
        "ALTER TABLE sessions ADD COLUMN harness_id TEXT;
         CREATE UNIQUE INDEX idx_sessions_harness_id ON sessions(harness_id) WHERE harness_id IS NOT NULL;",
    ),
    M::up(
        "CREATE TABLE session_messages (
            session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,
            id TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL,
            completed_at TEXT NOT NULL DEFAULT '',
            parent_id TEXT NOT NULL DEFAULT '',
            provider_id TEXT NOT NULL DEFAULT '',
            model_id TEXT NOT NULL DEFAULT '',
            error_json TEXT NOT NULL DEFAULT '',
            removed_at TEXT,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (session_id, id)
        );

        CREATE INDEX idx_session_messages_session_created_at
            ON session_messages(session_id, created_at, id);

        CREATE TABLE session_message_parts (
            session_id BLOB NOT NULL CHECK(length(session_id) = 16),
            message_id TEXT NOT NULL,
            id TEXT NOT NULL,
            part_type TEXT NOT NULL,
            text TEXT NOT NULL DEFAULT '',
            tool_json TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL,
            PRIMARY KEY (session_id, message_id, id),
            FOREIGN KEY (session_id, message_id) REFERENCES session_messages(session_id, id) ON DELETE CASCADE
        );

        CREATE INDEX idx_session_message_parts_session_message
            ON session_message_parts(session_id, message_id, id);",
    ),
];
pub const SQLITE_MIGRATIONS: Migrations<'_> = Migrations::from_slice(SQLITE_MIGRATIONS_SLICE);
