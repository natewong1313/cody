use chrono::{Duration, NaiveDateTime};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, sqlite::Sqlite},
    harness::Harness,
    harness::opencode::OpencodeHarness,
    repo::{
        assistant_message::AssistantMessage,
        message::{Message, MessageRepo, MessageRepoError},
        project::Project,
        session::Session,
        user_message::{UserMessage, UserMessagePart},
    },
};

use super::test_utils::{closed_port, fixed_datetime, wait_for_port};

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
        harness_session_id: format!("hs-{id}"),
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
        model_provider_id: "workers-ai".to_string(),
        model_id: "@cf/moonshotai/kimi-k2.5".to_string(),
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

fn user_message_part(
    id: Uuid,
    user_message_id: Uuid,
    session_id: Uuid,
    position: i64,
    text: &str,
    at: NaiveDateTime,
) -> UserMessagePart {
    UserMessagePart {
        id,
        user_message_id,
        session_id,
        position,
        part_type: "text".to_string(),
        text: Some(text.to_string()),
        file_name: None,
        file_url: None,
        agent_name: None,
        subtask_prompt: None,
        subtask_description: None,
        created_at: at,
        updated_at: at,
    }
}

async fn spawn_fake_opencode_server() -> (u32, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener bind should succeed");
    let port = listener
        .local_addr()
        .expect("listener local addr should exist")
        .port() as u32;

    let handle = tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };

            tokio::spawn(async move {
                let mut buf = [0_u8; 2048];
                let read = socket.read(&mut buf).await.unwrap_or_default();
                let request = String::from_utf8_lossy(&buf[..read]);
                let first_line = request.lines().next().unwrap_or_default();

                let response = if first_line.starts_with("POST /session/")
                    && first_line.contains("/prompt_async")
                {
                    "HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n".to_string()
                } else {
                    let body = r#"{"info":{"role":"assistant","id":"msg-assistant-1","sessionID":"ses-fake","time":{"created":1730000000000,"completed":1730000001000},"error":null,"parentID":"msg-user-1","modelID":"gpt-5","providerID":"openai","mode":"chat","path":{"cwd":"/tmp","root":"/tmp"},"cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"finish":"stop"},"parts":[{"id":"part-1","sessionID":"ses-fake","messageID":"msg-assistant-1","type":"text","text":"hello"}]}"#;
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                };

                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            });
        }
    });

    (port, handle)
}

#[tokio::test]
async fn create_user_message_persists_message() {
    let (port, server) = spawn_fake_opencode_server().await;
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let now = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    db.create_project(test_project(project_id, now))
        .await
        .expect("create project should succeed");
    db.create_session(test_session(session_id, project_id, now))
        .await
        .expect("create session should succeed");

    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(port));
    let repo = MessageRepo::new(ctx);

    let created = repo
        .create_user_message(
            user_message(message_id, session_id, now + Duration::seconds(1)),
            vec![user_message_part(
                Uuid::new_v4(),
                message_id,
                session_id,
                0,
                "hello",
                now + Duration::seconds(1),
            )],
        )
        .await
        .expect("create_user_message should succeed");

    assert_eq!(created.id, message_id);
    assert_eq!(created.session_id, session_id);

    let listed = repo
        .list_by_session(&session_id, 10)
        .await
        .expect("list_by_session should succeed");

    assert_eq!(listed.len(), 1);
    match &listed[0] {
        Message::User(m) => assert_eq!(m.id, message_id),
        _ => panic!("expected user message"),
    }

    server.abort();
}

#[tokio::test]
async fn create_user_message_maps_database_errors() {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let now = fixed_datetime();
    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(closed_port()));
    let repo = MessageRepo::new(ctx);

    let err = repo
        .create_user_message(user_message(Uuid::new_v4(), Uuid::new_v4(), now), vec![])
        .await
        .expect_err("create_user_message should fail for missing session");

    assert!(matches!(
        err,
        crate::backend::repo::message::MessageRepoError::SessionNotFound(_)
    ));
}

#[tokio::test]
async fn create_user_message_does_not_persist_when_harness_unavailable() {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let now = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    db.create_project(test_project(project_id, now))
        .await
        .expect("create project should succeed");
    db.create_session(test_session(session_id, project_id, now))
        .await
        .expect("create session should succeed");

    let ctx = BackendContext::new(db, OpencodeHarness::new_for_test(closed_port()));
    let repo = MessageRepo::new(ctx);

    let err = repo
        .create_user_message(
            user_message(message_id, session_id, now + Duration::seconds(1)),
            vec![user_message_part(
                Uuid::new_v4(),
                message_id,
                session_id,
                0,
                "hello",
                now + Duration::seconds(1),
            )],
        )
        .await
        .expect_err("create_user_message should fail when harness is unavailable");

    assert!(matches!(err, MessageRepoError::Harness(_)));

    let listed = repo
        .list_by_session(&session_id, 10)
        .await
        .expect("list_by_session should succeed");
    assert!(listed.is_empty());
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
// #[ignore = "requires local opencode binary"]
async fn create_user_message_sends_message_to_real_opencode() {
    let port = closed_port();
    let harness = OpencodeHarness::new_with_process_for_test(port)
        .expect("test harness with process should start");
    wait_for_port(port);

    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let now = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    let project_dir = std::env::temp_dir().join(format!("cody-opencode-test-{session_id}"));
    std::fs::create_dir_all(&project_dir).expect("project temp directory should be created");
    let project_dir_string = project_dir.to_string_lossy().to_string();

    db.create_project(Project {
        id: project_id,
        name: "proj".to_string(),
        dir: project_dir_string.clone(),
        created_at: now,
        updated_at: now,
    })
    .await
    .expect("create project should succeed");

    let harness_session_id = harness
        .create_session(
            test_session(session_id, project_id, now),
            Some(&project_dir_string),
        )
        .await
        .expect("create opencode session should succeed");

    let mut session = test_session(session_id, project_id, now);
    session.harness_session_id = harness_session_id.clone();
    session.dir = Some(project_dir_string.clone());
    db.create_session(session)
        .await
        .expect("create session should succeed");

    let harness_for_asserts = harness.clone();
    let ctx = BackendContext::new(db, harness);
    let repo = MessageRepo::new(ctx);

    let created = repo
        .create_user_message(
            user_message(message_id, session_id, now + Duration::seconds(1)),
            vec![user_message_part(
                Uuid::new_v4(),
                message_id,
                session_id,
                0,
                "hello from integration test",
                now + Duration::seconds(1),
            )],
        )
        .await
        .expect("create_user_message should succeed");

    assert_eq!(created.id, message_id);
    assert_eq!(created.session_id, session_id);

    let messages = harness_for_asserts
        .get_session_messages(&harness_session_id, Some(50), Some(&project_dir_string))
        .await
        .expect("get_session_messages should succeed");
    assert!(
        messages
            .iter()
            .all(|m| m.session_id() == harness_session_id),
        "all returned messages should belong to test harness session"
    );
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

    db.create_user_message(user_message(
        user_1_id,
        session_id,
        base + Duration::seconds(1),
    ))
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
    db.create_user_message(user_message(
        user_2_id,
        session_id,
        base + Duration::seconds(3),
    ))
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
    db.create_session(test_session(
        session_b,
        project_id,
        base + Duration::seconds(1),
    ))
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
    db.create_user_message(user_message(
        user_id,
        session_id,
        now + Duration::seconds(1),
    ))
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
