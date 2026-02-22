use chrono::Utc;
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    data::project::{Project, ProjectRepo, ProjectRepoError},
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

fn test_repo() -> ProjectRepo<Sqlite> {
    let db = Sqlite::new_in_memory().expect("in-memory db should initialize");
    let harness = OpencodeHarness::new_for_test(1);
    let ctx = BackendContext::new(db, harness);
    ProjectRepo::new(ctx)
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
