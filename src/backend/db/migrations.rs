use rusqlite_migration::{M, Migrations};

const SQLITE_MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "
CREATE TABLE projects (
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
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
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    project_id BLOB NOT NULL CHECK(length(project_id) = 16) REFERENCES projects(id) ON DELETE CASCADE,

    parent_session_id BLOB REFERENCES sessions(id) ON DELETE SET NULL,

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
CREATE TABLE messages (
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,

    parent_message_id BLOB REFERENCES messages(id) ON DELETE SET NULL,

    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),

    title TEXT,
    body TEXT, -- entire body in plaintext, made up of parts
    agent TEXT,
    system_message TEXT, -- system prompt
    variant TEXT, -- model variant

    is_finished_streaming INTEGER NOT NULL DEFAULT 0 CHECK(is_finished_streaming IN (0, 1)),
    is_summary INTEGER NOT NULL DEFAULT 0 CHECK(is_summary IN (0, 1)), -- if message is a summary artifact

    model_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,

    error_name TEXT,
    error_message TEXT,
    error_type TEXT,

    cwd TEXT,
    root TEXT,

    cost REAL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    reasoning_tokens INTEGER,
    cached_read_tokens INTEGER,
    cached_write_tokens INTEGER,
    total_tokens INTEGER,

    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX messages_session_created_idx ON messages(session_id, created_at);
CREATE INDEX messages_parent_message_id_idx ON messages(parent_message_id);
CREATE INDEX messages_role_idx ON messages(role);
",
    ),
    M::up(
        "
CREATE TABLE message_tools (
    message_id BLOB NOT NULL CHECK(length(message_id) = 16) REFERENCES messages(id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL, -- bash, read, edit, etc
    enabled INTEGER NOT NULL CHECK(enabled IN (0, 1)),
    PRIMARY KEY (message_id, tool_name) -- enforce one tool per message
);
CREATE INDEX message_tools_message_id_idx ON message_tools(message_id);
",
    ),
    M::up(
        "
CREATE TABLE message_parts (
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,
    message_id BLOB NOT NULL CHECK(length(message_id) = 16) REFERENCES messages(id) ON DELETE CASCADE,

    position INTEGER NOT NULL DEFAULT 0, -- used to properly order parts
    part_type TEXT NOT NULL,

    text_content TEXT,
    synthetic INTEGER CHECK(synthetic IN (0, 1)),
    ignored INTEGER CHECK(ignored IN (0, 1)),
    part_time_start TEXT,
    part_time_end TEXT,

    mime TEXT,
    filename TEXT,
    url TEXT,

    call_id TEXT,
    tool_name TEXT,
    tool_status TEXT,
    tool_title TEXT,
    tool_input_text TEXT,
    tool_output_text TEXT,
    tool_error_text TEXT,
    tool_time_start TEXT,
    tool_time_end TEXT,
    tool_time_compacted TEXT,

    step_reason TEXT,
    step_snapshot TEXT,
    step_cost REAL,
    step_input_tokens INTEGER,
    step_output_tokens INTEGER,
    step_reasoning_tokens INTEGER,
    step_cached_read_tokens INTEGER,
    step_cached_write_tokens INTEGER,
    step_total_tokens INTEGER,

    subtask_prompt TEXT,
    subtask_description TEXT,
    subtask_agent TEXT,
    subtask_model_provider_id TEXT,
    subtask_model_id TEXT,
    subtask_command TEXT,

    retry_attempt INTEGER,
    retry_error_message TEXT,
    retry_error_status_code INTEGER,
    retry_error_is_retryable INTEGER CHECK(retry_error_is_retryable IN (0, 1)),

    snapshot_ref TEXT,
    patch_hash TEXT,
    compaction_auto INTEGER CHECK(compaction_auto IN (0, 1)),
    agent_name TEXT,
    agent_source_value TEXT,
    agent_source_start INTEGER,
    agent_source_end INTEGER,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX message_parts_message_position_idx ON message_parts(message_id, position);
CREATE INDEX message_parts_session_created_idx ON message_parts(session_id, created_at);
CREATE INDEX message_parts_type_idx ON message_parts(part_type);
",
    ),
    M::up(
        "
CREATE TABLE message_part_attachments (
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    part_id BLOB NOT NULL CHECK(length(part_id) = 16) REFERENCES message_parts(id) ON DELETE CASCADE,
    mime TEXT NOT NULL,
    url TEXT NOT NULL,
    filename TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX message_part_attachments_part_id_idx ON message_part_attachments(part_id);

CREATE TABLE message_part_file_sources (
    part_id BLOB PRIMARY KEY NOT NULL CHECK(length(part_id) = 16) REFERENCES message_parts(id) ON DELETE CASCADE,

    source_type TEXT NOT NULL CHECK(source_type IN ('file', 'symbol', 'resource')),

    path TEXT,

    symbol_name TEXT,
    symbol_kind INTEGER,

    range_start_line INTEGER,
    range_start_col INTEGER,
    range_end_line INTEGER,
    range_end_col INTEGER,

    client_name TEXT,
    uri TEXT,

    source_text_value TEXT,
    source_text_start INTEGER,
    source_text_end INTEGER
);

CREATE TABLE message_part_patch_files (
    part_id BLOB NOT NULL CHECK(length(part_id) = 16)
        REFERENCES message_parts(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    PRIMARY KEY (part_id, file_path) -- ensures one row per file path per patch part
);

CREATE INDEX message_part_patch_files_part_id_idx ON message_part_patch_files(part_id);
",
    ),
    M::up(
        "
CREATE TABLE user_message (
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,

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
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    user_message_id BLOB NOT NULL CHECK(length(user_message_id) = 16) REFERENCES user_message(id) ON DELETE CASCADE,
    session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,

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
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,
    user_message_id BLOB NOT NULL CHECK(length(user_message_id) = 16) REFERENCES user_message(id) ON DELETE CASCADE,

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
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id) = 16),
    assistant_message_id BLOB NOT NULL CHECK(length(assistant_message_id) = 16)
        REFERENCES assistant_message(id) ON DELETE CASCADE,
    session_id BLOB NOT NULL CHECK(length(session_id) = 16) REFERENCES sessions(id) ON DELETE CASCADE,

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
];
pub const SQLITE_MIGRATIONS: Migrations<'_> = Migrations::from_slice(SQLITE_MIGRATIONS_SLICE);
