use chrono::Utc;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::watch,
};
use uuid::Uuid;

use crate::backend::{
    BackendContext, BackendService, Project, Session,
    db::sqlite::Sqlite,
    harness::opencode::OpencodeHarness,
    proto_project::ProjectModel,
    proto_session::SessionModel,
    repo::{project::ProjectRepo, session::SessionRepo},
};

pub fn closed_port() -> u32 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port bind should succeed");
    let port = listener
        .local_addr()
        .expect("listener local addr should exist")
        .port();
    drop(listener);
    port as u32
}

pub async fn spawn_fake_opencode_server() -> (u32, tokio::task::JoinHandle<()>) {
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

pub fn test_backend(port: u32) -> Arc<BackendService> {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let harness = OpencodeHarness::new_for_test(port);
    let ctx = BackendContext::new(db, harness);
    let (projects_sender, _) = watch::channel(Vec::new());

    Arc::new(BackendService {
        project_repo: ProjectRepo::new(ctx.clone()),
        projects_sender,
        session_repo: SessionRepo::new(ctx),
    })
}

pub fn test_project(name: &str, dir: &str) -> Project {
    let now = Utc::now().naive_utc();
    Project {
        id: Uuid::new_v4(),
        name: name.to_string(),
        dir: dir.to_string(),
        created_at: now,
        updated_at: now,
    }
}

pub fn test_session(project_id: Uuid, name: &str, show_in_gui: bool) -> Session {
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

pub fn valid_project_model() -> ProjectModel {
    ProjectModel {
        id: Uuid::new_v4().to_string(),
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        updated_at: "2025-01-02 03:04:05.123456".to_string(),
    }
}

pub fn valid_session_model(project_id: Uuid) -> SessionModel {
    SessionModel {
        id: Uuid::new_v4().to_string(),
        project_id: project_id.to_string(),
        show_in_gui: true,
        name: "sess".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        updated_at: "2025-01-02 03:04:05.123456".to_string(),
    }
}
