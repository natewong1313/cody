use crate::backend::db::Database;
use crate::backend::db::sqlite::Sqlite;
use crate::backend::repo::project::Project;
use chrono::Utc;
use uuid::Uuid;

fn create_test_project(name: &str, dir: &str) -> Project {
    let now = Utc::now().naive_utc();
    Project {
        id: Uuid::new_v4(),
        name: name.to_string(),
        dir: dir.to_string(),
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
async fn test_new_in_memory_creates_valid_database() {
    let result = Sqlite::new_in_memory();
    assert!(
        result.is_ok(),
        "Failed to create in-memory database: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_list_projects_returns_empty_vec_for_new_database() {
    let db = Sqlite::new_in_memory().unwrap();
    let projects = db.list_projects().await.unwrap();
    assert!(projects.is_empty());
}

#[tokio::test]
async fn test_create_and_get_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test Project", "/test/dir");

    let created = db.create_project(project.clone()).await.unwrap();
    assert_eq!(created.id, project.id);
    assert_eq!(created.name, project.name);
    assert_eq!(created.dir, project.dir);

    let retrieved = db.get_project(project.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, project.id);
    assert_eq!(retrieved.name, project.name);
}

#[tokio::test]
async fn test_get_project_returns_none_for_nonexistent() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.get_project(nonexistent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_project_with_empty_name_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("", "/test/dir");

    let result = db.create_project(project).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_project_with_whitespace_only_name_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("   ", "/test/dir");

    let result = db.create_project(project).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_project_with_empty_dir_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "");

    let result = db.create_project(project).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_project_with_whitespace_only_dir_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "   ");

    let result = db.create_project(project).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_duplicate_project_id_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");

    db.create_project(project.clone()).await.unwrap();

    let result = db.create_project(project).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Original", "/original/dir");
    let created = db.create_project(project).await.unwrap();

    let mut updated_project = created.clone();
    updated_project.name = "Updated".to_string();
    updated_project.dir = "/updated/dir".to_string();

    let updated = db.update_project(updated_project).await.unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.dir, "/updated/dir");
    assert_eq!(updated.id, created.id);
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_update_nonexistent_project_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");

    let result = db.update_project(project).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_project_with_empty_name_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created = db.create_project(project).await.unwrap();

    let mut updated = created.clone();
    updated.name = "".to_string();

    let result = db.update_project(updated).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created = db.create_project(project).await.unwrap();

    db.delete_project(created.id).await.unwrap();

    let result = db.get_project(created.id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_project_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.delete_project(nonexistent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_projects_ordered_by_updated_at_desc() {
    let db = Sqlite::new_in_memory().unwrap();

    let project1 = create_test_project("Project 1", "/dir1");
    let created1 = db.create_project(project1).await.unwrap();

    let project2 = create_test_project("Project 2", "/dir2");
    let created2 = db.create_project(project2).await.unwrap();

    let project3 = create_test_project("Project 3", "/dir3");
    let created3 = db.create_project(project3).await.unwrap();

    let mut updated2 = created2.clone();
    updated2.name = "Updated Project 2".to_string();
    db.update_project(updated2).await.unwrap();

    let projects = db.list_projects().await.unwrap();
    assert_eq!(projects.len(), 3);
    assert_eq!(projects[0].id, created2.id);
    assert_eq!(projects[1].id, created3.id);
    assert_eq!(projects[2].id, created1.id);
}

#[tokio::test]
async fn test_project_with_special_characters_in_name() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project(
        "Test \"Project\" with 'quotes' and \\backslash",
        "/test/dir",
    );
    let created = db.create_project(project.clone()).await.unwrap();

    let retrieved = db.get_project(created.id).await.unwrap().unwrap();
    assert_eq!(retrieved.name, project.name);
}

#[tokio::test]
async fn test_project_with_unicode_name() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("テストプロジェクト 🚀 Ñoño", "/test/dir");
    let created = db.create_project(project.clone()).await.unwrap();

    let retrieved = db.get_project(created.id).await.unwrap().unwrap();
    assert_eq!(retrieved.name, project.name);
}

#[tokio::test]
async fn test_multiple_sequential_operations() {
    let db = Sqlite::new_in_memory().unwrap();

    for i in 0..10 {
        let project = create_test_project(&format!("Project {i}"), &format!("/dir/{i}"));
        db.create_project(project).await.unwrap();
    }

    let projects = db.list_projects().await.unwrap();
    assert_eq!(projects.len(), 10);
}

#[tokio::test]
async fn test_update_project_preserves_created_at() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created = db.create_project(project).await.unwrap();
    let original_created_at = created.created_at;

    let mut updated = created.clone();
    updated.name = "Updated".to_string();
    let result = db.update_project(updated).await.unwrap();

    assert_eq!(result.created_at, original_created_at);
    assert!(result.updated_at > original_created_at);
}

#[tokio::test]
async fn test_create_project_with_very_long_name() {
    let db = Sqlite::new_in_memory().unwrap();
    let long_name = "a".repeat(1000);
    let project = create_test_project(&long_name, "/test/dir");

    let created = db.create_project(project.clone()).await.unwrap();
    assert_eq!(created.name, long_name);
}
