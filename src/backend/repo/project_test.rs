use chrono::{NaiveDateTime, Utc};
use tonic::Code;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::sqlite::Sqlite,
    harness::opencode::OpencodeHarness,
    proto_project::ProjectModel,
    repo::project::{Project, ProjectRepo, ProjectRepoError},
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

fn test_repo() -> ProjectRepo<Sqlite> {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let harness = OpencodeHarness::new_for_test(1);
    let ctx = BackendContext::new(db, harness);
    ProjectRepo::new(ctx)
}

fn fixed_datetime() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2025-01-02 03:04:05.123456", "%Y-%m-%d %H:%M:%S%.f")
        .expect("fixed datetime should parse")
}

#[test]
fn project_proto_serialize_to_model() {
    let id = Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("uuid should parse");
    let ts = fixed_datetime();
    let project = Project {
        id,
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: ts,
        updated_at: ts,
    };

    let model: ProjectModel = project.into();

    assert_eq!(model.id, "11111111-2222-3333-4444-555555555555");
    assert_eq!(model.name, "proj");
    assert_eq!(model.dir, "/tmp/proj");
    assert_eq!(model.created_at, "2025-01-02 03:04:05.123456");
    assert_eq!(model.updated_at, "2025-01-02 03:04:05.123456");
}

#[test]
fn project_proto_deserialize_from_model() {
    let model = ProjectModel {
        id: "11111111-2222-3333-4444-555555555555".to_string(),
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        updated_at: "2025-01-02 03:04:05.123456".to_string(),
    };

    let project = Project::try_from(model).expect("valid project model should deserialize");

    assert_eq!(
        project.id,
        Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("uuid should parse")
    );
    assert_eq!(project.name, "proj");
    assert_eq!(project.dir, "/tmp/proj");
    assert_eq!(project.created_at, fixed_datetime());
    assert_eq!(project.updated_at, fixed_datetime());
}

#[test]
fn project_proto_deserialize_rejects_invalid_uuid() {
    let model = ProjectModel {
        id: "not-a-uuid".to_string(),
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        updated_at: "2025-01-02 03:04:05.123456".to_string(),
    };

    let err = Project::try_from(model).expect_err("invalid uuid should fail");
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("project.id"));
}

#[test]
fn project_proto_deserialize_rejects_invalid_datetime() {
    let model = ProjectModel {
        id: "11111111-2222-3333-4444-555555555555".to_string(),
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: "not-a-datetime".to_string(),
        updated_at: "2025-01-02 03:04:05.123456".to_string(),
    };

    let err = Project::try_from(model).expect_err("invalid datetime should fail");
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("project.created_at"));
}

#[tokio::test]
async fn list_is_empty_for_new_repo() {
    let repo = test_repo();
    let projects = repo.list().await.expect("list should succeed");
    assert!(projects.is_empty());
}

#[tokio::test]
async fn create_and_get_project() {
    let repo = test_repo();
    let project = test_project("proj", "/tmp/proj");

    let created = repo.create(&project).await.expect("create should succeed");
    let fetched = repo.get(&created.id).await.expect("get should succeed");

    assert!(fetched.is_some());
    let fetched = fetched.expect("project should exist");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "proj");
    assert_eq!(fetched.dir, "/tmp/proj");
}

#[tokio::test]
async fn get_returns_none_for_missing_project() {
    let repo = test_repo();
    let missing = Uuid::new_v4();

    let fetched = repo.get(&missing).await.expect("get should succeed");
    assert!(fetched.is_none());
}

#[tokio::test]
async fn update_project() {
    let repo = test_repo();
    let created = repo
        .create(&test_project("orig", "/tmp/orig"))
        .await
        .expect("create should succeed");

    let mut updated = created.clone();
    updated.name = "updated".to_string();
    updated.dir = "/tmp/updated".to_string();

    let result = repo.update(&updated).await.expect("update should succeed");
    assert_eq!(result.id, created.id);
    assert_eq!(result.name, "updated");
    assert_eq!(result.dir, "/tmp/updated");
    assert!(result.updated_at > created.updated_at);
}

#[tokio::test]
async fn delete_project() {
    let repo = test_repo();
    let created = repo
        .create(&test_project("delete-me", "/tmp/delete-me"))
        .await
        .expect("create should succeed");

    repo.delete(&created.id)
        .await
        .expect("delete should succeed");

    let fetched = repo.get(&created.id).await.expect("get should succeed");
    assert!(fetched.is_none());
}

#[tokio::test]
async fn create_maps_database_errors() {
    let repo = test_repo();
    let invalid = test_project("", "/tmp/proj");

    let err = repo
        .create(&invalid)
        .await
        .expect_err("invalid create should fail");

    assert!(matches!(err, ProjectRepoError::Database(_)));
}
