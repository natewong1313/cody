use chrono::{Duration, NaiveDateTime};
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, sqlite::Sqlite},
    harness::opencode::OpencodeHarness,
    repo::{
        assistant_message::AssistantMessage,
        message::{Message, MessageRepo},
        project::Project,
        session::Session,
        user_message::UserMessage,
    },
};

fn fixed_datetime() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2025-01-02 03:04:05.123456", "%Y-%m-%d %H:%M:%S%.f")
        .expect("fixed datetime should parse")
}

fn closed_port() -> u32 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port bind should succeed");
    let port = listener
        .local_addr()
        .expect("listener local addr should exist")
        .port();
    drop(listener);
    port as u32
}

fn test_project(id: Uuid, at: NaiveDateTime) -> Project {
    Project {
        id,
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: at,
        updated_at: at,
    }
}

fn test_session(id: Uuid, project_id: Uuid, at: NaiveDateTime) -> Session {
    Session {
        id,
        project_id,
        parent_session_id: None,
        show_in_gui: true,
        name: "sess".to_string(),
        harness_type: "opencode".to_string(),
        harness_session_id: None,
        dir: Some("/tmp/proj".to_string()),
        summary_additions: None,
        summary_deletions: None,
        summary_files: None,
        created_at: at,
        updated_at: at,
    }
}

fn user_message(id: Uuid, session_id: Uuid, at: NaiveDateTime) -> UserMessage {
    UserMessage {
        id,
        session_id,
        agent: "build".to_string(),
        model_provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        system_prompt: None,
        structured_output_type: "text".to_string(),
        tools_list: "{}".to_string(),
        thinking_variant: None,
        created_at: at,
        updated_at: at,
    }
}

fn assistant_message(
    id: Uuid,
    session_id: Uuid,
    user_message_id: Uuid,
    at: NaiveDateTime,
) -> AssistantMessage {
    AssistantMessage {
        id,
        session_id,
        user_message_id,
        agent: "build".to_string(),
        model_provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        cwd: "/tmp/proj".to_string(),
        root: "/tmp/proj".to_string(),
        cost: 0.0,
        token_total: Some(3),
        token_input: 1,
        token_output: 2,
        token_reasoning: 0,
        token_cache_read: 0,
        token_cache_write: 0,
        error_message: None,
        created_at: at,
        updated_at: at,
        completed_at: Some(at),
    }
}

#[tokio::test]
async fn list_by_session_returns_empty_for_new_session() {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let now = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    db.create_project(test_project(project_id, now))
        .await
        .expect("create project should succeed");
    db.create_session(test_session(session_id, project_id, now))
        .await
        .expect("create session should succeed");

    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(closed_port()));
    let repo = MessageRepo::new(ctx);

    let out = repo
        .list_by_session(&session_id, 10)
        .await
        .expect("list_by_session should succeed");
    assert!(out.is_empty());
}

#[tokio::test]
async fn list_by_session_returns_latest_n_mixed_messages() {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let base = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    db.create_project(test_project(project_id, base))
        .await
        .expect("create project should succeed");
    db.create_session(test_session(session_id, project_id, base))
        .await
        .expect("create session should succeed");

    let user_1_id = Uuid::new_v4();
    let assistant_1_id = Uuid::new_v4();
    let user_2_id = Uuid::new_v4();
    let assistant_2_id = Uuid::new_v4();

    db.create_user_message(user_message(user_1_id, session_id, base + Duration::seconds(1)))
        .await
        .expect("create user message 1 should succeed");
    db.create_assistant_message(assistant_message(
        assistant_1_id,
        session_id,
        user_1_id,
        base + Duration::seconds(2),
    ))
    .await
    .expect("create assistant message 1 should succeed");
    db.create_user_message(user_message(user_2_id, session_id, base + Duration::seconds(3)))
        .await
        .expect("create user message 2 should succeed");
    db.create_assistant_message(assistant_message(
        assistant_2_id,
        session_id,
        user_2_id,
        base + Duration::seconds(4),
    ))
    .await
    .expect("create assistant message 2 should succeed");

    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(closed_port()));
    let repo = MessageRepo::new(ctx);

    let out = repo
        .list_by_session(&session_id, 3)
        .await
        .expect("list_by_session should succeed");

    assert_eq!(out.len(), 3);
    match &out[0] {
        Message::Assistant(m) => assert_eq!(m.id, assistant_1_id),
        _ => panic!("expected assistant message at index 0"),
    }
    match &out[1] {
        Message::User(m) => assert_eq!(m.id, user_2_id),
        _ => panic!("expected user message at index 1"),
    }
    match &out[2] {
        Message::Assistant(m) => assert_eq!(m.id, assistant_2_id),
        _ => panic!("expected assistant message at index 2"),
    }
}

#[tokio::test]
async fn list_by_session_filters_to_requested_session() {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let base = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();

    db.create_project(test_project(project_id, base))
        .await
        .expect("create project should succeed");
    db.create_session(test_session(session_a, project_id, base))
        .await
        .expect("create session a should succeed");
    db.create_session(test_session(session_b, project_id, base + Duration::seconds(1)))
        .await
        .expect("create session b should succeed");

    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();

    db.create_user_message(user_message(user_a, session_a, base + Duration::seconds(2)))
        .await
        .expect("create session a message should succeed");
    db.create_user_message(user_message(user_b, session_b, base + Duration::seconds(3)))
        .await
        .expect("create session b message should succeed");

    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(closed_port()));
    let repo = MessageRepo::new(ctx);

    let out = repo
        .list_by_session(&session_a, 10)
        .await
        .expect("list_by_session should succeed");

    assert_eq!(out.len(), 1);
    match &out[0] {
        Message::User(m) => assert_eq!(m.id, user_a),
        _ => panic!("expected user message"),
    }
}

#[tokio::test]
async fn list_by_session_zero_limit_returns_empty() {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let now = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    db.create_project(test_project(project_id, now))
        .await
        .expect("create project should succeed");
    db.create_session(test_session(session_id, project_id, now))
        .await
        .expect("create session should succeed");
    db.create_user_message(user_message(user_id, session_id, now + Duration::seconds(1)))
        .await
        .expect("create user message should succeed");

    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(closed_port()));
    let repo = MessageRepo::new(ctx);

    let out = repo
        .list_by_session(&session_id, 0)
        .await
        .expect("list_by_session should succeed");
    assert!(out.is_empty());
}
