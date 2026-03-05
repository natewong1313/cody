use rusqlite_migration::{M, Migrations};

const SQLITE_MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "
CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    dir TEXT NOT NULL CHECK(length(trim(dir)) > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX projects_dir_idx ON projects(dir);
",
    ),
    M::up(
        "
CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,

    parent_session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,

    show_in_gui INTEGER NOT NULL DEFAULT 0 CHECK(show_in_gui IN (0, 1)),

    name TEXT NOT NULL DEFAULT 'New Session',
    harness_type TEXT NOT NULL DEFAULT 'opencode',
    harness_session_id TEXT NOT NULL,

    dir TEXT,
    summary_additions INTEGER,
    summary_deletions INTEGER,
    summary_files INTEGER,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX sessions_harness_session_id_uq ON sessions(harness_session_id);
CREATE INDEX sessions_project_id_idx ON sessions(project_id);
CREATE INDEX sessions_parent_session_id_idx ON sessions(parent_session_id);
",
    ),
    M::up(
        "
CREATE TABLE user_message (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,

    agent TEXT NOT NULL DEFAULT 'build',
    model_provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    system_prompt TEXT,
    structured_output_type TEXT NOT NULL DEFAULT 'text' CHECK(structured_output_type IN ('text', 'json')),
    tools_list TEXT NOT NULL DEFAULT '{}' CHECK(json_valid(tools_list)),
    thinking_variant TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX user_message_session_created_idx ON user_message(session_id, created_at);

CREATE TABLE user_message_part (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    user_message_id TEXT NOT NULL REFERENCES user_message(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,

    position INTEGER NOT NULL,
    part_type TEXT NOT NULL,

    text TEXT,
    file_name TEXT,
    file_url TEXT,
    agent_name TEXT,
    subtask_prompt TEXT,
    subtask_description TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    UNIQUE(user_message_id, position)
);
CREATE INDEX user_message_part_message_position_idx ON user_message_part(user_message_id, position);
CREATE INDEX user_message_part_session_created_idx ON user_message_part(session_id, created_at);
CREATE INDEX user_message_part_type_idx ON user_message_part(part_type);

CREATE TABLE assistant_message (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_message_id TEXT NOT NULL REFERENCES user_message(id) ON DELETE CASCADE,

    agent TEXT NOT NULL,
    model_provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,

    cwd TEXT NOT NULL,
    root TEXT NOT NULL,

    cost REAL NOT NULL DEFAULT 0,

    token_total INTEGER,
    token_input INTEGER NOT NULL DEFAULT 0,
    token_output INTEGER NOT NULL DEFAULT 0,
    token_reasoning INTEGER NOT NULL DEFAULT 0,
    token_cache_read INTEGER NOT NULL DEFAULT 0,
    token_cache_write INTEGER NOT NULL DEFAULT 0,

    error_message TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT
);
CREATE INDEX assistant_message_session_created_idx ON assistant_message(session_id, created_at);
CREATE INDEX assistant_message_user_message_id_idx ON assistant_message(user_message_id);

CREATE TABLE assistant_message_part (
    id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
    assistant_message_id TEXT NOT NULL
        REFERENCES assistant_message(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,

    position INTEGER NOT NULL,
    part_type TEXT NOT NULL,

    text TEXT,

    file_mime TEXT,
    file_filename TEXT,
    file_url TEXT,
    file_source_type TEXT,
    file_source_path TEXT,
    file_source_name TEXT,
    file_source_kind INTEGER,
    file_source_uri TEXT,
    file_source_text_value TEXT,
    file_source_text_start INTEGER,
    file_source_text_end INTEGER,

    agent_name TEXT,
    subtask_prompt TEXT,
    subtask_description TEXT,
    subtask_agent TEXT,
    subtask_model_provider_id TEXT,
    subtask_model_id TEXT,
    subtask_command TEXT,

    tool_call_id TEXT,
    tool_name TEXT,
    tool_status TEXT CHECK(tool_status IN ('pending', 'running', 'completed', 'error')),
    tool_input_json TEXT,
    tool_output_text TEXT,
    tool_error_text TEXT,
    tool_title TEXT,
    tool_metadata_json TEXT,
    tool_compacted_at INTEGER,
    tool_state_raw TEXT,
    tool_state_time_start INTEGER,
    tool_state_time_end INTEGER,
    tool_state_time_compacted INTEGER,
    tool_attachments_json TEXT,

    finish_reason TEXT,
    cost REAL,
    token_total INTEGER,
    token_input INTEGER,
    token_output INTEGER,
    token_reasoning INTEGER,
    token_cache_read INTEGER,
    token_cache_write INTEGER,

    snapshot_hash TEXT,
    patch_hash TEXT,
    patch_files_json TEXT,
    retry_attempt INTEGER,
    retry_error_json TEXT,
    retry_created_at INTEGER,

    compaction_auto INTEGER CHECK(compaction_auto IN (0, 1)),

    delta_field TEXT,
    delta_text TEXT,
    part_time_start INTEGER,
    part_time_end INTEGER,
    part_metadata_json TEXT,
    text_synthetic INTEGER CHECK(text_synthetic IN (0, 1)),
    text_ignored INTEGER CHECK(text_ignored IN (0, 1)),
    step_snapshot_hash TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    UNIQUE(assistant_message_id, position)
);
CREATE INDEX assistant_message_part_message_position_idx ON assistant_message_part(assistant_message_id, position);
CREATE INDEX assistant_message_part_session_created_idx ON assistant_message_part(session_id, created_at);
CREATE INDEX assistant_message_part_type_idx ON assistant_message_part(part_type);
CREATE INDEX assistant_message_part_tool_call_id_idx ON assistant_message_part(tool_call_id);
",
    ),
    M::up(
        "
ALTER TABLE assistant_message ADD COLUMN harness_message_id TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS assistant_message_session_harness_message_id_uq
    ON assistant_message(session_id, harness_message_id)
    WHERE harness_message_id IS NOT NULL;

ALTER TABLE assistant_message_part ADD COLUMN harness_part_id TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS assistant_message_part_message_harness_part_id_uq
    ON assistant_message_part(assistant_message_id, harness_part_id)
    WHERE harness_part_id IS NOT NULL;
",
    ),
];
pub const SQLITE_MIGRATIONS: Migrations<'_> = Migrations::from_slice(SQLITE_MIGRATIONS_SLICE);
