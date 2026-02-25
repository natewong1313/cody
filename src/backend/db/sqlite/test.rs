use crate::backend::db::Database;
use crate::backend::db::sqlite::Sqlite;
use crate::backend::repo::{
    message::{Message, MessagePart},
    project::Project,
    session::Session,
};
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

fn create_test_session(project_id: Uuid, name: &str, show_in_gui: bool) -> Session {
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

fn create_test_message(session_id: Uuid, id: &str) -> Message {
    Message {
        id: id.to_string(),
        session_id,
        role: "assistant".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        completed_at: "2025-01-02 03:04:06.123456".to_string(),
        parent_id: "parent-1".to_string(),
        provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        error_json: String::new(),
        parts: vec![MessagePart {
            id: "part-1".to_string(),
            message_id: id.to_string(),
            part_type: "text".to_string(),
            text: "hello".to_string(),
            tool_json: String::new(),
        }],
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
async fn test_create_and_get_session() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test Project", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session = create_test_session(created_project.id, "Test Session", true);
    let created = db.create_session(session.clone()).await.unwrap();

    assert_eq!(created.id, session.id);
    assert_eq!(created.project_id, session.project_id);
    assert_eq!(created.name, session.name);
    assert_eq!(created.show_in_gui, session.show_in_gui);

    let retrieved = db.get_session(session.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, session.id);
}

#[tokio::test]
async fn test_get_session_returns_none_for_nonexistent() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.get_session(nonexistent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_session_for_nonexistent_project_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_project_id = Uuid::new_v4();

    let session = create_test_session(nonexistent_project_id, "Test Session", true);
    let result = db.create_session(session).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_duplicate_session_id_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session = create_test_session(created_project.id, "Test Session", true);
    db.create_session(session.clone()).await.unwrap();

    let result = db.create_session(session).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_sessions_by_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session1 = create_test_session(created_project.id, "Session 1", true);
    let session2 = create_test_session(created_project.id, "Session 2", false);
    let created1 = db.create_session(session1).await.unwrap();
    let created2 = db.create_session(session2).await.unwrap();

    let sessions = db
        .list_sessions_by_project(created_project.id)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, created2.id);
    assert_eq!(sessions[1].id, created1.id);
}

#[tokio::test]
async fn test_list_sessions_by_project_returns_empty_for_nonexistent() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_project_id = Uuid::new_v4();

    let sessions = db
        .list_sessions_by_project(nonexistent_project_id)
        .await
        .unwrap();
    assert!(sessions.is_empty());
}

#[tokio::test]
async fn test_list_sessions_ordered_by_updated_at_desc() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session1 = create_test_session(created_project.id, "Session 1", true);
    let created1 = db.create_session(session1).await.unwrap();

    let session2 = create_test_session(created_project.id, "Session 2", true);
    let created2 = db.create_session(session2).await.unwrap();

    let session3 = create_test_session(created_project.id, "Session 3", true);
    let created3 = db.create_session(session3).await.unwrap();

    let mut updated1 = created1.clone();
    updated1.name = "Updated Session 1".to_string();
    db.update_session(updated1).await.unwrap();

    let sessions = db
        .list_sessions_by_project(created_project.id)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].id, created1.id);
    assert_eq!(sessions[1].id, created3.id);
    assert_eq!(sessions[2].id, created2.id);
}

#[tokio::test]
async fn test_update_session() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session = create_test_session(created_project.id, "Original", true);
    let created = db.create_session(session).await.unwrap();

    let mut updated_session = created.clone();
    updated_session.name = "Updated".to_string();
    updated_session.show_in_gui = false;

    let updated = db.update_session(updated_session).await.unwrap();
    assert_eq!(updated.name, "Updated");
    assert!(!updated.show_in_gui);
    assert_eq!(updated.id, created.id);
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_update_session_change_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project1 = create_test_project("Project 1", "/dir1");
    let project2 = create_test_project("Project 2", "/dir2");
    let created_project1 = db.create_project(project1).await.unwrap();
    let created_project2 = db.create_project(project2).await.unwrap();

    let session = create_test_session(created_project1.id, "Test", true);
    let created = db.create_session(session).await.unwrap();

    let mut updated = created.clone();
    updated.project_id = created_project2.id;

    let result = db.update_session(updated).await.unwrap();
    assert_eq!(result.project_id, created_project2.id);
}

#[tokio::test]
async fn test_update_nonexistent_session_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session = create_test_session(created_project.id, "Test", true);

    let result = db.update_session(session).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_session() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session = create_test_session(created_project.id, "Test", true);
    let created = db.create_session(session).await.unwrap();

    db.delete_session(created.id).await.unwrap();

    let result = db.get_session(created.id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_session_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.delete_session(nonexistent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cascade_delete_sessions_when_project_deleted() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session1 = create_test_session(created_project.id, "Session 1", true);
    let session2 = create_test_session(created_project.id, "Session 2", true);
    let created_session1 = db.create_session(session1).await.unwrap();
    let created_session2 = db.create_session(session2).await.unwrap();

    db.delete_project(created_project.id).await.unwrap();

    let session1_result = db.get_session(created_session1.id).await.unwrap();
    let session2_result = db.get_session(created_session2.id).await.unwrap();
    assert!(session1_result.is_none());
    assert!(session2_result.is_none());
}

#[tokio::test]
async fn test_multiple_projects_and_sessions() {
    let db = Sqlite::new_in_memory().unwrap();

    let project1 = create_test_project("Project 1", "/dir1");
    let project2 = create_test_project("Project 2", "/dir2");
    let created_project1 = db.create_project(project1).await.unwrap();
    let created_project2 = db.create_project(project2).await.unwrap();

    for i in 0..5 {
        let session = create_test_session(created_project1.id, &format!("Session {i}"), i % 2 == 0);
        db.create_session(session).await.unwrap();
    }

    for i in 0..3 {
        let session = create_test_session(created_project2.id, &format!("Session {i}"), true);
        db.create_session(session).await.unwrap();
    }

    let projects = db.list_projects().await.unwrap();
    assert_eq!(projects.len(), 2);

    let sessions1 = db
        .list_sessions_by_project(created_project1.id)
        .await
        .unwrap();
    assert_eq!(sessions1.len(), 5);

    let sessions2 = db
        .list_sessions_by_project(created_project2.id)
        .await
        .unwrap();
    assert_eq!(sessions2.len(), 3);
}

#[tokio::test]
async fn test_session_show_in_gui_default() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let now = Utc::now().naive_utc();
    let session = Session {
        id: Uuid::new_v4(),
        project_id: created_project.id,
        show_in_gui: false,
        name: "Test".to_string(),
        created_at: now,
        updated_at: now,
    };

    let created = db.create_session(session).await.unwrap();
    assert!(!created.show_in_gui);
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
async fn test_update_session_preserves_created_at() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).await.unwrap();

    let session = create_test_session(created_project.id, "Test", true);
    let created = db.create_session(session).await.unwrap();
    let original_created_at = created.created_at;

    let mut updated = created.clone();
    updated.name = "Updated".to_string();
    let result = db.update_session(updated).await.unwrap();

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

#[tokio::test]
async fn test_session_isolation_between_projects() {
    let db = Sqlite::new_in_memory().unwrap();
    let project1 = create_test_project("Project 1", "/dir1");
    let project2 = create_test_project("Project 2", "/dir2");
    let created_project1 = db.create_project(project1).await.unwrap();
    let created_project2 = db.create_project(project2).await.unwrap();

    let session1 = create_test_session(created_project1.id, "Session 1", true);
    let session2 = create_test_session(created_project2.id, "Session 2", true);
    let created_session1 = db.create_session(session1).await.unwrap();
    let created_session2 = db.create_session(session2).await.unwrap();

    let sessions1 = db
        .list_sessions_by_project(created_project1.id)
        .await
        .unwrap();
    assert_eq!(sessions1.len(), 1);
    assert_eq!(sessions1[0].id, created_session1.id);

    let sessions2 = db
        .list_sessions_by_project(created_project2.id)
        .await
        .unwrap();
    assert_eq!(sessions2.len(), 1);
    assert_eq!(sessions2[0].id, created_session2.id);
}

#[tokio::test]
async fn test_session_harness_id_round_trip() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = db
        .create_project(create_test_project("Test", "/test/dir"))
        .await
        .unwrap();
    let session = db
        .create_session(create_test_session(project.id, "Test", true))
        .await
        .unwrap();

    db.set_session_harness_id(session.id, "ses-abc".to_string())
        .await
        .unwrap();

    let by_session = db.get_session_harness_id(session.id).await.unwrap();
    assert_eq!(by_session.as_deref(), Some("ses-abc"));

    let reverse = db.get_session_id_by_harness_id("ses-abc").await.unwrap();
    assert_eq!(reverse, Some(session.id));
}

#[tokio::test]
async fn test_message_upsert_and_list() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = db
        .create_project(create_test_project("Test", "/test/dir"))
        .await
        .unwrap();
    let session = db
        .create_session(create_test_session(project.id, "Test", true))
        .await
        .unwrap();

    let message = create_test_message(session.id, "msg-1");
    db.upsert_session_message(message.clone()).await.unwrap();
    for part in &message.parts {
        db.upsert_session_message_part(session.id, part.clone(), None)
            .await
            .unwrap();
    }

    let listed = db.list_session_messages(session.id, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "msg-1");
    assert_eq!(listed[0].parts.len(), 1);
    assert_eq!(listed[0].parts[0].text, "hello");
}

#[tokio::test]
async fn test_message_mark_removed_excludes_from_list() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = db
        .create_project(create_test_project("Test", "/test/dir"))
        .await
        .unwrap();
    let session = db
        .create_session(create_test_session(project.id, "Test", true))
        .await
        .unwrap();

    let message = create_test_message(session.id, "msg-removed");
    db.upsert_session_message(message.clone()).await.unwrap();
    for part in &message.parts {
        db.upsert_session_message_part(session.id, part.clone(), None)
            .await
            .unwrap();
    }

    db.mark_session_message_removed(session.id, "msg-removed")
        .await
        .unwrap();

    let listed = db.list_session_messages(session.id, None).await.unwrap();
    assert!(listed.is_empty());
}

#[tokio::test]
async fn test_ensure_session_message_exists_allows_part_before_message() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = db
        .create_project(create_test_project("Test", "/test/dir"))
        .await
        .unwrap();
    let session = db
        .create_session(create_test_session(project.id, "Test", true))
        .await
        .unwrap();

    db.ensure_session_message_exists(session.id, "msg-orphan")
        .await
        .unwrap();
    db.upsert_session_message_part(
        session.id,
        MessagePart {
            id: "part-1".to_string(),
            message_id: "msg-orphan".to_string(),
            part_type: "text".to_string(),
            text: "hello".to_string(),
            tool_json: String::new(),
        },
        None,
    )
    .await
    .unwrap();

    let listed = db.list_session_messages(session.id, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "msg-orphan");
    assert_eq!(listed[0].parts.len(), 1);
    assert_eq!(listed[0].parts[0].text, "hello");
}

#[tokio::test]
async fn test_message_part_duplicate_delta_does_not_double_append() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = db
        .create_project(create_test_project("Test", "/test/dir"))
        .await
        .unwrap();
    let session = db
        .create_session(create_test_session(project.id, "Test", true))
        .await
        .unwrap();

    db.ensure_session_message_exists(session.id, "msg-delta")
        .await
        .unwrap();

    let part = MessagePart {
        id: "part-1".to_string(),
        message_id: "msg-delta".to_string(),
        part_type: "text".to_string(),
        text: String::new(),
        tool_json: String::new(),
    };

    db.upsert_session_message_part(session.id, part.clone(), Some("hello".to_string()))
        .await
        .unwrap();
    db.upsert_session_message_part(session.id, part, Some("hello".to_string()))
        .await
        .unwrap();

    let listed = db.list_session_messages(session.id, None).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].parts.len(), 1);
    assert_eq!(listed[0].parts[0].text, "hello");
}

#[tokio::test]
async fn test_upsert_message_with_parts_rolls_back_on_part_error() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = db
        .create_project(create_test_project("Test", "/test/dir"))
        .await
        .unwrap();
    let session = db
        .create_session(create_test_session(project.id, "Test", true))
        .await
        .unwrap();

    let message = Message {
        id: "msg-rollback".to_string(),
        session_id: session.id,
        role: "assistant".to_string(),
        created_at: "2025-01-02 03:04:05.123456".to_string(),
        completed_at: "2025-01-02 03:04:06.123456".to_string(),
        parent_id: String::new(),
        provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        error_json: String::new(),
        parts: vec![MessagePart {
            id: "part-invalid".to_string(),
            message_id: "different-message-id".to_string(),
            part_type: "text".to_string(),
            text: "hello".to_string(),
            tool_json: String::new(),
        }],
    };

    let err = db.upsert_session_message_with_parts(message).await;
    assert!(
        err.is_err(),
        "expected FK error from invalid part message_id"
    );

    let listed = db.list_session_messages(session.id, None).await.unwrap();
    assert!(listed.is_empty(), "message insert should be rolled back");
}
