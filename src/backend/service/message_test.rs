use futures::StreamExt;
use tonic::Request;

use crate::backend::{
    proto_message::{
        ListSessionMessagesRequest, MessageInput, MessagePartInput, SendMessageRequest,
        SubscribeSessionMessagesRequest, message_server::Message as MessageService,
    },
    proto_session::{CreateSessionRequest, session_server::Session as SessionService},
    service::test_helpers::{
        spawn_fake_opencode_server, test_backend, test_project, valid_session_model,
    },
};

#[tokio::test]
async fn send_message_happy_path() {
    let (port, server) = spawn_fake_opencode_server().await;
    let backend = test_backend(port);

    let project = backend
        .project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");

    let session = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(valid_session_model(project.id)),
        }))
        .await
        .expect("create_session should succeed")
        .into_inner()
        .session
        .expect("session should be present");

    let reply = backend
        .send_message(Request::new(SendMessageRequest {
            session_id: session.id,
            input: Some(MessageInput {
                parts: vec![MessagePartInput {
                    text: "hello".to_string(),
                    synthetic: false,
                    ignored: false,
                }],
                message_id: String::new(),
                agent: String::new(),
                no_reply: false,
                system: String::new(),
                model: None,
            }),
        }))
        .await
        .expect("send_message should succeed")
        .into_inner();

    let message = reply.message.expect("message should be present");
    assert_eq!(message.role, "assistant");
    assert!(!message.parts.is_empty());

    server.abort();
}

#[tokio::test]
async fn list_session_messages_happy_path() {
    let (port, server) = spawn_fake_opencode_server().await;
    let backend = test_backend(port);

    let project = backend
        .project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");

    let session = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(valid_session_model(project.id)),
        }))
        .await
        .expect("create_session should succeed")
        .into_inner()
        .session
        .expect("session should be present");

    backend
        .send_message(Request::new(SendMessageRequest {
            session_id: session.id.clone(),
            input: Some(MessageInput {
                parts: vec![MessagePartInput {
                    text: "hello".to_string(),
                    synthetic: false,
                    ignored: false,
                }],
                message_id: String::new(),
                agent: String::new(),
                no_reply: false,
                system: String::new(),
                model: None,
            }),
        }))
        .await
        .expect("send_message should succeed");

    let reply = backend
        .list_session_messages(Request::new(ListSessionMessagesRequest {
            session_id: session.id,
            limit: None,
        }))
        .await
        .expect("list_session_messages should succeed")
        .into_inner();

    assert!(!reply.messages.is_empty());
    assert!(reply.messages.iter().any(|m| m.role == "assistant"));

    server.abort();
}

#[tokio::test]
async fn subscribe_session_messages_receives_updates() {
    let (port, server) = spawn_fake_opencode_server().await;
    let backend = test_backend(port);

    let project = backend
        .project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");

    let session = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(valid_session_model(project.id)),
        }))
        .await
        .expect("create_session should succeed")
        .into_inner()
        .session
        .expect("session should be present");

    let response = backend
        .subscribe_session_messages(Request::new(SubscribeSessionMessagesRequest {
            session_id: session.id.clone(),
        }))
        .await
        .expect("subscribe_session_messages should succeed");

    let mut stream = response.into_inner();
    let initial = stream
        .next()
        .await
        .expect("initial stream item should exist")
        .expect("initial stream item should be ok");
    assert!(initial.messages.is_empty());

    backend
        .send_message(Request::new(SendMessageRequest {
            session_id: session.id,
            input: Some(MessageInput {
                parts: vec![MessagePartInput {
                    text: "hello".to_string(),
                    synthetic: false,
                    ignored: false,
                }],
                message_id: String::new(),
                agent: String::new(),
                no_reply: false,
                system: String::new(),
                model: None,
            }),
        }))
        .await
        .expect("send_message should succeed");

    let update = stream
        .next()
        .await
        .expect("update stream item should exist")
        .expect("update stream item should be ok");
    assert!(!update.messages.is_empty());

    server.abort();
}

#[tokio::test]
async fn list_session_messages_rejects_non_positive_limit() {
    let backend = test_backend(1);

    let err = backend
        .list_session_messages(Request::new(ListSessionMessagesRequest {
            session_id: "11111111-2222-3333-4444-555555555555".to_string(),
            limit: Some(0),
        }))
        .await
        .expect_err("non-positive limit should fail");

    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("limit must be greater than 0"));
}
