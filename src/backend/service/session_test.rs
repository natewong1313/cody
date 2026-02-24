use tonic::{Code, Request};
use uuid::Uuid;

use crate::backend::{
    session::{
        CreateSessionRequest, DeleteSessionRequest, GetSessionRequest, ListSessionsByProjectRequest,
        UpdateSessionRequest, session_server::Session as SessionService,
    },
    service::test_helpers::{
        closed_port, spawn_fake_opencode_server, test_backend, test_project, test_session,
        valid_session_model,
    },
};

#[tokio::test]
async fn list_sessions_by_project_returns_only_matching_project() {
    let (port, server) = spawn_fake_opencode_server().await;
    let backend = test_backend(port);

    let p1 = backend
        .project_repo
        .create(&test_project("p1", "/tmp/p1"))
        .await
        .expect("project create should succeed");
    let p2 = backend
        .project_repo
        .create(&test_project("p2", "/tmp/p2"))
        .await
        .expect("project create should succeed");

    backend
        .session_repo
        .create(&test_session(p1.id, "s1", true))
        .await
        .expect("session create should succeed");
    backend
        .session_repo
        .create(&test_session(p1.id, "s2", false))
        .await
        .expect("session create should succeed");
    backend
        .session_repo
        .create(&test_session(p2.id, "other", true))
        .await
        .expect("session create should succeed");

    let response = backend
        .list_sessions_by_project(Request::new(ListSessionsByProjectRequest {
            project_id: p1.id.to_string(),
        }))
        .await
        .expect("list_sessions_by_project should succeed");
    let sessions = response.into_inner().sessions;

    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().all(|s| s.project_id == p1.id.to_string()));

    server.abort();
}

#[tokio::test]
async fn list_sessions_by_project_rejects_invalid_project_id() {
    let backend = test_backend(closed_port());

    let err = backend
        .list_sessions_by_project(Request::new(ListSessionsByProjectRequest {
            project_id: "bad-uuid".to_string(),
        }))
        .await
        .expect_err("invalid id should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("project_id"));
}

#[tokio::test]
async fn get_session_rejects_invalid_session_id() {
    let backend = test_backend(closed_port());

    let err = backend
        .get_session(Request::new(GetSessionRequest {
            session_id: "bad-uuid".to_string(),
        }))
        .await
        .expect_err("invalid id should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("session_id"));
}

#[tokio::test]
async fn create_session_rejects_missing_session_field() {
    let backend = test_backend(closed_port());

    let err = backend
        .create_session(Request::new(CreateSessionRequest { session: None }))
        .await
        .expect_err("missing session should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("missing session"));
}

#[tokio::test]
async fn create_session_happy_path() {
    let (port, server) = spawn_fake_opencode_server().await;
    let backend = test_backend(port);

    let project = backend
        .project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");

    let created = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(valid_session_model(project.id)),
        }))
        .await
        .expect("create_session should succeed")
        .into_inner()
        .session
        .expect("created session should exist");

    assert_eq!(created.project_id, project.id.to_string());
    assert_eq!(created.name, "sess");

    server.abort();
}

#[tokio::test]
async fn create_session_returns_not_found_for_missing_project() {
    let backend = test_backend(closed_port());
    let session = valid_session_model(Uuid::new_v4());

    let err = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(session),
        }))
        .await
        .expect_err("missing project should fail");

    assert_eq!(err.code(), Code::NotFound);
    assert!(err.message().contains("project not found"));
}

#[tokio::test]
async fn create_session_returns_unavailable_when_harness_fails() {
    let backend = test_backend(closed_port());
    let project = backend
        .project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");

    let err = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(valid_session_model(project.id)),
        }))
        .await
        .expect_err("closed harness port should fail");

    assert_eq!(err.code(), Code::Unavailable);
}

#[tokio::test]
async fn update_and_delete_session_happy_path() {
    let (port, server) = spawn_fake_opencode_server().await;
    let backend = test_backend(port);
    let project = backend
        .project_repo
        .create(&test_project("proj", "/tmp/proj"))
        .await
        .expect("project create should succeed");

    let created = backend
        .create_session(Request::new(CreateSessionRequest {
            session: Some(valid_session_model(project.id)),
        }))
        .await
        .expect("create_session should succeed")
        .into_inner()
        .session
        .expect("created session should exist");

    let mut updated = created.clone();
    updated.name = "updated".to_string();
    updated.show_in_gui = false;

    let updated_response = backend
        .update_session(Request::new(UpdateSessionRequest {
            session: Some(updated),
        }))
        .await
        .expect("update_session should succeed")
        .into_inner()
        .session
        .expect("updated session should exist");

    assert_eq!(updated_response.name, "updated");
    assert!(!updated_response.show_in_gui);

    backend
        .delete_session(Request::new(DeleteSessionRequest {
            session_id: updated_response.id.clone(),
        }))
        .await
        .expect("delete_session should succeed");

    let fetched = backend
        .get_session(Request::new(GetSessionRequest {
            session_id: updated_response.id,
        }))
        .await
        .expect("get_session should succeed")
        .into_inner()
        .session;

    assert!(fetched.is_none());

    server.abort();
}
