use chrono::Utc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use uuid::Uuid;

use crate::backend::{
    BackendContext, Project,
    data::{
        project::ProjectRepo,
        session::{Session, SessionRepo, SessionRepoError},
    },
    db::sqlite::Sqlite,
    harness::opencode::OpencodeHarness,
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

fn test_repos(port: u32) -> (ProjectRepo<Sqlite>, SessionRepo<Sqlite>) {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let harness = OpencodeHarness::new_for_test(port);
    let ctx = BackendContext::new(db, harness);
    (ProjectRepo::new(ctx.clone()), SessionRepo::new(ctx))
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
                let _ = socket.read(&mut buf).await;

                let body = r#"{"id":"fake-session-id","title":"fake"}"#;
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

#[tokio::test]
async fn list_by_project_returns_only_project_sessions() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo) = test_repos(port);

    let p1 = project_repo
        .create(&test_project("p1", "/tmp/p1"))
        .await
        .expect("project create should succeed");
    let p2 = project_repo
        .create(&test_project("p2", "/tmp/p2"))
        .await
        .expect("project create should succeed");

    let created_p1s1 = session_repo
        .create(&test_session(p1.id, "s1", true))
        .await
        .expect("create session should succeed");
    let created_p1s2 = session_repo
        .create(&test_session(p1.id, "s2", false))
        .await
        .expect("create session should succeed");
    session_repo
        .create(&test_session(p2.id, "other", true))
        .await
        .expect("create session should succeed");

    let sessions = session_repo
        .list_by_project(&p1.id)
        .await
        .expect("list_by_project should succeed");

    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().any(|s| s.id == created_p1s1.id));
    assert!(sessions.iter().any(|s| s.id == created_p1s2.id));
    assert!(sessions.iter().all(|s| s.project_id == p1.id));

    server.abort();
}

#[tokio::test]
async fn get_returns_none_for_missing_session() {
    let (_project_repo, session_repo) = test_repos(closed_port());
    let missing = Uuid::new_v4();

    let fetched = session_repo
        .get(&missing)
        .await
        .expect("get should succeed");
    assert!(fetched.is_none());
}

#[tokio::test]
async fn create_returns_project_not_found_when_project_missing() {
    let (_project_repo, session_repo) = test_repos(closed_port());
    let missing_project = Uuid::new_v4();
    let session = test_session(missing_project, "missing-project", true);

    let err = session_repo
        .create(&session)
        .await
        .expect_err("create should fail");

    assert!(matches!(err, SessionRepoError::ProjectNotFound(id) if id == missing_project));
}

#[tokio::test]
async fn create_rolls_back_when_harness_fails() {
    let (project_repo, session_repo) = test_repos(closed_port());
    let project = project_repo
        .create(&test_project("p", "/tmp/p"))
        .await
        .expect("project create should succeed");
    let session = test_session(project.id, "will-fail", true);

    let err = session_repo
        .create(&session)
        .await
        .expect_err("harness should fail on closed port");
    assert!(matches!(err, SessionRepoError::Harness(_)));

    let fetched = session_repo
        .get(&session.id)
        .await
        .expect("get should succeed");
    assert!(fetched.is_none());
}

#[tokio::test]
async fn update_and_delete_session() {
    let (port, server) = spawn_fake_opencode_server().await;
    let (project_repo, session_repo) = test_repos(port);
    let project = project_repo
        .create(&test_project("p", "/tmp/p"))
        .await
        .expect("project create should succeed");

    let seeded = session_repo
        .create(&test_session(project.id, "seed", true))
        .await
        .expect("create session should succeed");

    let mut updated = seeded.clone();
    updated.name = "updated".to_string();
    updated.show_in_gui = false;

    let result = session_repo
        .update(&updated)
        .await
        .expect("update should succeed");
    assert_eq!(result.id, seeded.id);
    assert_eq!(result.name, "updated");
    assert!(!result.show_in_gui);

    session_repo
        .delete(&result.id)
        .await
        .expect("delete should succeed");

    let fetched = session_repo
        .get(&result.id)
        .await
        .expect("get should succeed");
    assert!(fetched.is_none());

    server.abort();
}
