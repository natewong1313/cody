use chrono::Utc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use uuid::Uuid;

use crate::backend::{
    BackendContext, Project, Session,
    db::sqlite::Sqlite,
    harness::{
        ModelSelection, OpencodeGlobalEvent, OpencodePartInput, OpencodeSendMessageRequest,
        opencode::OpencodeHarness,
    },
    proto_message::{MessageInput, MessageModel, MessagePartInput, MessagePartModel},
    repo::{
        message::{Message, MessagePart, MessageRepo, MessageRepoError},
        project::ProjectRepo,
        session::SessionRepo,
    },
};

fn test_project(name: &str, dir: &str) -> Project {
    let now = Utc::now().naive_utc();
    Project {
        id: Uuid::new_v4(),
        name: name.to_string(),
        dir: dir.to_string(),
        created_at: now,
        updated_at: now,
    }
}

fn test_session(project_id: Uuid, name: &str, show_in_gui: bool) -> Session {
    let now = Utc::now().naive_utc();
    Session {
        id: Uuid::new_v4(),
        project_id,
        show_in_gui,
        name: name.to_string(),
        created_at: now,
        updated_at: now,
    }
}

fn test_repos(
    port: u32,
) -> (
    ProjectRepo<Sqlite>,
    SessionRepo<Sqlite>,
    MessageRepo<Sqlite>,
) {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let harness = OpencodeHarness::new_for_test(port);
    let ctx = BackendContext::new(db, harness);
    (
        ProjectRepo::new(ctx.clone()),
        SessionRepo::new(ctx.clone()),
        MessageRepo::new(ctx),
    )
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

fn valid_input(text: &str) -> MessageInput {
    MessageInput {
        parts: vec![MessagePartInput {
            text: text.to_string(),
            synthetic: false,
            ignored: false,
        }],
        message_id: String::new(),
        agent: String::new(),
        no_reply: false,
        system: String::new(),
        model: None,
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

                let body = if first_line.starts_with("POST /session/")
                    && first_line.contains("/message")
                {
                    r#"{"info":{"role":"assistant","id":"msg-assistant-1","sessionID":"ses-fake","time":{"created":1730000000000,"completed":1730000001000},"error":null,"parentID":"msg-user-1","modelID":"gpt-5","providerID":"openai","mode":"chat","path":{"cwd":"/tmp","root":"/tmp"},"cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"finish":"stop"},"parts":[{"id":"part-1","sessionID":"ses-fake","messageID":"msg-assistant-1","type":"text","text":"hello"}]}"#.to_string()
                } else if first_line.starts_with("GET /session/") && first_line.contains("/message")
                {
                    r#"[{"info":{"role":"user","id":"msg-user-1","sessionID":"ses-fake","time":{"created":1730000000000},"summary":null,"agent":"build","model":{"providerID":"openai","modelID":"gpt-5"},"system":null,"tools":null},"parts":[{"id":"part-user-1","sessionID":"ses-fake","messageID":"msg-user-1","type":"text","text":"hi"}]},{"info":{"role":"assistant","id":"msg-assistant-1","sessionID":"ses-fake","time":{"created":1730000000000,"completed":1730000001000},"error":null,"parentID":"msg-user-1","modelID":"gpt-5","providerID":"openai","mode":"chat","path":{"cwd":"/tmp","root":"/tmp"},"cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"finish":"stop"},"parts":[{"id":"part-1","sessionID":"ses-fake","messageID":"msg-assistant-1","type":"text","text":"hello"}]}]"#.to_string()
                } else {
                    "{\"id\":\"ses-fake\",\"title\":\"fake\"}".to_string()
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );

                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            });
        }
    });

    (port, handle)
}

#[test]
fn message_part_proto_serialize_to_model() {
    let part = MessagePart {
        id: "part-1".to_string(),
        message_id: "msg-1".to_string(),
        part_type: "text".to_string(),
        text: "hello".to_string(),
        tool_json: String::new(),
    };

    let model: MessagePartModel = part.into();

    assert_eq!(model.id, "part-1");
    assert_eq!(model.message_id, "msg-1");
    assert_eq!(model.r#type, "text");
    assert_eq!(model.text, "hello");
    assert!(model.tool_json.is_empty());
}

#[test]
fn message_proto_serialize_to_model() {
    let session_id =
        Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").expect("uuid should parse");
    let message = Message {
        id: "msg-1".to_string(),
        session_id,
        role: "assistant".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        completed_at: "2025-01-02 03:04:06.123456".to_string(),
        parent_id: "msg-parent".to_string(),
        provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        error_json: String::new(),
        parts: vec![MessagePart {
            id: "part-1".to_string(),
            message_id: "msg-1".to_string(),
            part_type: "text".to_string(),
            text: "hello".to_string(),
            tool_json: String::new(),
        }],
    };

    let model: MessageModel = message.into();

    assert_eq!(model.id, "msg-1");
    assert_eq!(model.session_id, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
    assert_eq!(model.role, "assistant");
    assert_eq!(model.created_at, "2025-01-02 03:04:05.123456");
    assert_eq!(model.completed_at, "2025-01-02 03:04:06.123456");
    assert_eq!(model.parent_id, "msg-parent");
    assert_eq!(model.provider_id, "openai");
    assert_eq!(model.model_id, "gpt-5");
    assert_eq!(model.parts.len(), 1);
    assert_eq!(model.parts[0].id, "part-1");
}

#[test]
fn message_input_maps_to_send_request() {
    let input = MessageInput {
        parts: vec![MessagePartInput {
            text: "hello".to_string(),
            synthetic: true,
            ignored: true,
        }],
        message_id: "  msg-1  ".to_string(),
        agent: "  build  ".to_string(),
        no_reply: true,
        system: "  be concise  ".to_string(),
        model: Some(crate::backend::proto_message::ModelSelection {
            provider_id: "openai".to_string(),
            model_id: "gpt-5".to_string(),
        }),
    };

    let req = OpencodeSendMessageRequest::try_from(input)
        .expect("valid message input should map to send request");

    assert_eq!(req.message_id.as_deref(), Some("msg-1"));
    assert_eq!(req.agent.as_deref(), Some("build"));
    assert_eq!(req.system.as_deref(), Some("be concise"));
    assert_eq!(req.no_reply, Some(true));

    let model: Option<ModelSelection> = req.model;
    let model = model.expect("model should be present");
    assert_eq!(model.provider_id, "openai");
    assert_eq!(model.model_id, "gpt-5");

    assert_eq!(req.parts.len(), 1);
    match &req.parts[0] {
        OpencodePartInput::Text {
            id,
            text,
            synthetic,
            ignored,
        } => {
            assert!(id.is_none());
            assert_eq!(text, "hello");
            assert_eq!(*synthetic, Some(true));
            assert_eq!(*ignored, Some(true));
        }
        _ => panic!("expected text part input"),
    }
}

#[test]
fn message_input_rejects_empty_parts() {
    let input = MessageInput {
        parts: Vec::new(),
        message_id: String::new(),
        agent: String::new(),
        no_reply: false,
        system: String::new(),
        model: None,
    };

    let err = OpencodeSendMessageRequest::try_from(input).expect_err("empty parts should fail");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("at least one part"));
}

#[test]
fn message_input_rejects_blank_text_part() {
    let input = MessageInput {
        parts: vec![MessagePartInput {
            text: "   ".to_string(),
            synthetic: false,
            ignored: false,
        }],
        message_id: String::new(),
        agent: String::new(),
        no_reply: false,
        system: String::new(),
        model: None,
    };

    let err = OpencodeSendMessageRequest::try_from(input).expect_err("blank text part should fail");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("cannot be empty"));
}

fn event_from_json(json: &str) -> OpencodeGlobalEvent {
    serde_json::from_str(json).expect("event json should deserialize")
}

#[tokio::test]
async fn list_messages_returns_session_not_found_for_missing_session() {
    let (_project_repo, _session_repo, message_repo) = test_repos(closed_port());
    let missing = Uuid::new_v4();

    let err = message_repo
        .list_messages(&missing, None)
        .await
        .expect_err("missing session should fail");

    assert!(matches!(err, MessageRepoError::SessionNotFound(id) if id == missing));
}

#[tokio::test]
async fn send_message_happy_path_persists_message() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    let sent = message_repo
        .send_message(&session.id, valid_input("hello"))
        .await
        .expect("send_message should succeed");

    assert_eq!(sent.role, "assistant");
    assert!(!sent.parts.is_empty());

    let listed = message_repo
        .list_messages(&session.id, None)
        .await
        .expect("list_messages should succeed");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "msg-assistant-1");
    assert_eq!(listed[0].parts.len(), 1);
    assert_eq!(listed[0].parts[0].text, "hello");

    server.abort();
}

#[tokio::test]
async fn send_message_rejects_invalid_input() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    let err = message_repo
        .send_message(
            &session.id,
            MessageInput {
                parts: Vec::new(),
                message_id: String::new(),
                agent: String::new(),
                no_reply: false,
                system: String::new(),
                model: None,
            },
        )
        .await
        .expect_err("empty parts should fail");

    assert!(matches!(err, MessageRepoError::InvalidInput(_)));

    server.abort();
}

#[tokio::test]
async fn send_message_returns_harness_error_when_server_unavailable() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    server.abort();

    let err = message_repo
        .send_message(&session.id, valid_input("hello"))
        .await
        .expect_err("send_message should fail when harness is unavailable");
    assert!(matches!(err, MessageRepoError::Harness(_)));
}

#[tokio::test]
async fn reconcile_session_messages_persists_harness_messages() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    message_repo
        .reconcile_session_messages(&session.id, None)
        .await
        .expect("reconcile_session_messages should succeed");

    let listed = message_repo
        .list_messages(&session.id, None)
        .await
        .expect("list_messages should succeed");

    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|m| m.role == "user"));
    assert!(listed.iter().any(|m| m.role == "assistant"));

    server.abort();
}

#[tokio::test]
async fn ingest_event_returns_none_when_harness_session_not_mapped() {
    let (_project_repo, _session_repo, message_repo) = test_repos(closed_port());

    let event = event_from_json(
        r#"{
            "directory": "/tmp",
            "payload": {
                "type": "session.idle",
                "properties": { "sessionID": "ses-unknown" }
            }
        }"#,
    );

    let ingested = message_repo
        .ingest_event(event)
        .await
        .expect("ingest_event should succeed");
    assert!(ingested.is_none());
}

#[tokio::test]
async fn ingest_event_message_updated_persists_message() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    let event = event_from_json(
        r#"{
            "directory": "/tmp/proj",
            "payload": {
                "type": "message.updated",
                "properties": {
                    "info": {
                        "role": "user",
                        "id": "msg-event-user-1",
                        "sessionID": "ses-fake",
                        "time": { "created": 1730000000000 },
                        "summary": null,
                        "agent": "build",
                        "model": { "providerID": "openai", "modelID": "gpt-5" },
                        "system": null,
                        "tools": null
                    }
                }
            }
        }"#,
    );

    let ingested = message_repo
        .ingest_event(event)
        .await
        .expect("ingest_event should succeed");
    assert_eq!(ingested, Some(session.id));

    let listed = message_repo
        .list_messages(&session.id, None)
        .await
        .expect("list_messages should succeed");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "msg-event-user-1");
    assert_eq!(listed[0].role, "user");
    assert!(listed[0].parts.is_empty());

    server.abort();
}

#[tokio::test]
async fn ingest_event_part_updated_upserts_message_part_with_delta() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    let event = event_from_json(
        r#"{
            "directory": "/tmp/proj",
            "payload": {
                "type": "message.part.updated",
                "properties": {
                    "part": {
                        "type": "text",
                        "id": "part-event-1",
                        "sessionID": "ses-fake",
                        "messageID": "msg-event-assistant-1",
                        "text": "",
                        "synthetic": null,
                        "ignored": null
                    },
                    "delta": "hello"
                }
            }
        }"#,
    );

    let ingested = message_repo
        .ingest_event(event)
        .await
        .expect("ingest_event should succeed");
    assert_eq!(ingested, Some(session.id));

    let event = event_from_json(
        r#"{
            "directory": "/tmp/proj",
            "payload": {
                "type": "message.part.updated",
                "properties": {
                    "part": {
                        "type": "text",
                        "id": "part-event-1",
                        "sessionID": "ses-fake",
                        "messageID": "msg-event-assistant-1",
                        "text": "",
                        "synthetic": null,
                        "ignored": null
                    },
                    "delta": "hello"
                }
            }
        }"#,
    );

    let ingested = message_repo
        .ingest_event(event)
        .await
        .expect("second ingest_event should succeed");
    assert_eq!(ingested, Some(session.id));

    let listed = message_repo
        .list_messages(&session.id, None)
        .await
        .expect("list_messages should succeed");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "msg-event-assistant-1");
    assert_eq!(listed[0].parts.len(), 1);
    assert_eq!(listed[0].parts[0].id, "part-event-1");
    assert_eq!(listed[0].parts[0].text, "hello");

    server.abort();
}

#[tokio::test]
async fn ingest_event_message_removed_marks_message_removed() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    message_repo
        .send_message(&session.id, valid_input("hello"))
        .await
        .expect("send_message should succeed");

    let event = event_from_json(
        r#"{
            "directory": "/tmp/proj",
            "payload": {
                "type": "message.removed",
                "properties": {
                    "sessionID": "ses-fake",
                    "messageID": "msg-assistant-1"
                }
            }
        }"#,
    );

    let ingested = message_repo
        .ingest_event(event)
        .await
        .expect("ingest_event should succeed");
    assert_eq!(ingested, Some(session.id));

    let listed = message_repo
        .list_messages(&session.id, None)
        .await
        .expect("list_messages should succeed");
    assert!(listed.is_empty());

    server.abort();
}

#[tokio::test]
async fn ingest_event_session_idle_reconciles_messages() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo, message_repo) = test_repos(port);

    let project = project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");
    let session = session_repo
        .create(&test_session(project.id, "sess", true))
        .await
        .expect("session create should succeed");

    let event = event_from_json(
        r#"{
            "directory": "/tmp/proj",
            "payload": {
                "type": "session.idle",
                "properties": { "sessionID": "ses-fake" }
            }
        }"#,
    );

    let ingested = message_repo
        .ingest_event(event)
        .await
        .expect("ingest_event should succeed");
    assert_eq!(ingested, Some(session.id));

    let listed = message_repo
        .list_messages(&session.id, None)
        .await
        .expect("list_messages should succeed");
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|m| m.role == "user"));
    assert!(listed.iter().any(|m| m.role == "assistant"));

    server.abort();
}
