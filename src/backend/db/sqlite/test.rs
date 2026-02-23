use crate::backend::data::{project::Project, session::Session};
use crate::backend::db::sqlite::Sqlite;
use crate::backend::db::{Database, DatabaseTransaction};
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
    let projects = db.list_projects().unwrap();
    assert!(projects.is_empty());
}

#[tokio::test]
async fn test_create_and_get_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test Project", "/test/dir");

    let created = db.create_project(project.clone()).unwrap();
    assert_eq!(created.id, project.id);
    assert_eq!(created.name, project.name);
    assert_eq!(created.dir, project.dir);

    let retrieved = db.get_project(project.id).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, project.id);
    assert_eq!(retrieved.name, project.name);
}

#[tokio::test]
async fn test_get_project_returns_none_for_nonexistent() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.get_project(nonexistent_id).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_project_with_empty_name_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("", "/test/dir");

    let result = db.create_project(project);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_project_with_whitespace_only_name_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("   ", "/test/dir");

    let result = db.create_project(project);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_project_with_empty_dir_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "");

    let result = db.create_project(project);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_project_with_whitespace_only_dir_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "   ");

    let result = db.create_project(project);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_duplicate_project_id_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");

    db.create_project(project.clone()).unwrap();

    let result = db.create_project(project);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Original", "/original/dir");
    let created = db.create_project(project).unwrap();

    let mut updated_project = created.clone();
    updated_project.name = "Updated".to_string();
    updated_project.dir = "/updated/dir".to_string();

    let updated = db.update_project(updated_project).unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.dir, "/updated/dir");
    assert_eq!(updated.id, created.id);
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_update_nonexistent_project_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");

    let result = db.update_project(project);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_project_with_empty_name_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created = db.create_project(project).unwrap();

    let mut updated = created.clone();
    updated.name = "".to_string();

    let result = db.update_project(updated);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created = db.create_project(project).unwrap();

    db.delete_project(created.id).unwrap();

    let result = db.get_project(created.id).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_project_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.delete_project(nonexistent_id);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_projects_ordered_by_updated_at_desc() {
    let db = Sqlite::new_in_memory().unwrap();

    let project1 = create_test_project("Project 1", "/dir1");
    let created1 = db.create_project(project1).unwrap();

    let project2 = create_test_project("Project 2", "/dir2");
    let created2 = db.create_project(project2).unwrap();

    let project3 = create_test_project("Project 3", "/dir3");
    let created3 = db.create_project(project3).unwrap();

    let mut updated2 = created2.clone();
    updated2.name = "Updated Project 2".to_string();
    db.update_project(updated2).unwrap();

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 3);
    assert_eq!(projects[0].id, created2.id);
    assert_eq!(projects[1].id, created3.id);
    assert_eq!(projects[2].id, created1.id);
}

#[tokio::test]
async fn test_create_and_get_session() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test Project", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session = create_test_session(created_project.id, "Test Session", true);
    let created = db.create_session(session.clone()).unwrap();

    assert_eq!(created.id, session.id);
    assert_eq!(created.project_id, session.project_id);
    assert_eq!(created.name, session.name);
    assert_eq!(created.show_in_gui, session.show_in_gui);

    let retrieved = db.get_session(session.id).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, session.id);
}

#[tokio::test]
async fn test_get_session_returns_none_for_nonexistent() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.get_session(nonexistent_id).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_session_for_nonexistent_project_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_project_id = Uuid::new_v4();

    let session = create_test_session(nonexistent_project_id, "Test Session", true);
    let result = db.create_session(session);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_duplicate_session_id_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session = create_test_session(created_project.id, "Test Session", true);
    db.create_session(session.clone()).unwrap();

    let result = db.create_session(session);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_sessions_by_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session1 = create_test_session(created_project.id, "Session 1", true);
    let session2 = create_test_session(created_project.id, "Session 2", false);
    let created1 = db.create_session(session1).unwrap();
    let created2 = db.create_session(session2).unwrap();

    let sessions = db.list_sessions_by_project(created_project.id).unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, created2.id);
    assert_eq!(sessions[1].id, created1.id);
}

#[tokio::test]
async fn test_list_sessions_by_project_returns_empty_for_nonexistent() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_project_id = Uuid::new_v4();

    let sessions = db.list_sessions_by_project(nonexistent_project_id).unwrap();
    assert!(sessions.is_empty());
}

#[tokio::test]
async fn test_list_sessions_ordered_by_updated_at_desc() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session1 = create_test_session(created_project.id, "Session 1", true);
    let created1 = db.create_session(session1).unwrap();

    let session2 = create_test_session(created_project.id, "Session 2", true);
    let created2 = db.create_session(session2).unwrap();

    let session3 = create_test_session(created_project.id, "Session 3", true);
    let created3 = db.create_session(session3).unwrap();

    let mut updated1 = created1.clone();
    updated1.name = "Updated Session 1".to_string();
    db.update_session(updated1).unwrap();

    let sessions = db.list_sessions_by_project(created_project.id).unwrap();
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].id, created1.id);
    assert_eq!(sessions[1].id, created3.id);
    assert_eq!(sessions[2].id, created2.id);
}

#[tokio::test]
async fn test_update_session() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session = create_test_session(created_project.id, "Original", true);
    let created = db.create_session(session).unwrap();

    let mut updated_session = created.clone();
    updated_session.name = "Updated".to_string();
    updated_session.show_in_gui = false;

    let updated = db.update_session(updated_session).unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.show_in_gui, false);
    assert_eq!(updated.id, created.id);
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_update_session_change_project() {
    let db = Sqlite::new_in_memory().unwrap();
    let project1 = create_test_project("Project 1", "/dir1");
    let project2 = create_test_project("Project 2", "/dir2");
    let created_project1 = db.create_project(project1).unwrap();
    let created_project2 = db.create_project(project2).unwrap();

    let session = create_test_session(created_project1.id, "Test", true);
    let created = db.create_session(session).unwrap();

    let mut updated = created.clone();
    updated.project_id = created_project2.id;

    let result = db.update_session(updated).unwrap();
    assert_eq!(result.project_id, created_project2.id);
}

#[tokio::test]
async fn test_update_nonexistent_session_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session = create_test_session(created_project.id, "Test", true);

    let result = db.update_session(session);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_session() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session = create_test_session(created_project.id, "Test", true);
    let created = db.create_session(session).unwrap();

    db.delete_session(created.id).unwrap();

    let result = db.get_session(created.id).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_session_fails() {
    let db = Sqlite::new_in_memory().unwrap();
    let nonexistent_id = Uuid::new_v4();

    let result = db.delete_session(nonexistent_id);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cascade_delete_sessions_when_project_deleted() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session1 = create_test_session(created_project.id, "Session 1", true);
    let session2 = create_test_session(created_project.id, "Session 2", true);
    let created_session1 = db.create_session(session1).unwrap();
    let created_session2 = db.create_session(session2).unwrap();

    db.delete_project(created_project.id).unwrap();

    let session1_result = db.get_session(created_session1.id).unwrap();
    let session2_result = db.get_session(created_session2.id).unwrap();
    assert!(session1_result.is_none());
    assert!(session2_result.is_none());
}

#[tokio::test]
async fn test_multiple_projects_and_sessions() {
    let db = Sqlite::new_in_memory().unwrap();

    let project1 = create_test_project("Project 1", "/dir1");
    let project2 = create_test_project("Project 2", "/dir2");
    let created_project1 = db.create_project(project1).unwrap();
    let created_project2 = db.create_project(project2).unwrap();

    for i in 0..5 {
        let session =
            create_test_session(created_project1.id, &format!("Session {}", i), i % 2 == 0);
        db.create_session(session).unwrap();
    }

    for i in 0..3 {
        let session = create_test_session(created_project2.id, &format!("Session {}", i), true);
        db.create_session(session).unwrap();
    }

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 2);

    let sessions1 = db.list_sessions_by_project(created_project1.id).unwrap();
    assert_eq!(sessions1.len(), 5);

    let sessions2 = db.list_sessions_by_project(created_project2.id).unwrap();
    assert_eq!(sessions2.len(), 3);
}

#[tokio::test]
async fn test_session_show_in_gui_default() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let now = Utc::now().naive_utc();
    let session = Session {
        id: Uuid::new_v4(),
        project_id: created_project.id,
        show_in_gui: false,
        name: "Test".to_string(),
        created_at: now,
        updated_at: now,
    };

    let created = db.create_session(session).unwrap();
    assert_eq!(created.show_in_gui, false);
}

#[tokio::test]
async fn test_transaction_commit_persists_writes() {
    let db = Sqlite::new_in_memory().unwrap();

    let mut tx = db.begin_transaction().unwrap();

    let project = create_test_project("Project Tx", "/tx/dir");
    let mut created_project =
        <Sqlite as Database>::create_project(&db, project, Some(&mut tx)).unwrap();

    created_project.name = "Project Tx Updated".to_string();
    let updated_project =
        <Sqlite as Database>::update_project(&db, created_project, Some(&mut tx)).unwrap();

    let session = create_test_session(updated_project.id, "Session Tx", true);
    let mut created_session =
        <Sqlite as Database>::create_session(&db, session, Some(&mut tx)).unwrap();

    created_session.name = "Session Tx Updated".to_string();
    let updated_session =
        <Sqlite as Database>::update_session(&db, created_session, Some(&mut tx)).unwrap();

    <Sqlite as Database>::delete_session(&db, updated_session.id, Some(&mut tx)).unwrap();
    <Sqlite as Database>::delete_project(&db, updated_project.id, Some(&mut tx)).unwrap();

    tx.commit().unwrap();
    drop(tx);

    let project_result = db.get_project(updated_project.id).unwrap();
    let session_result = db.get_session(updated_session.id).unwrap();
    assert!(project_result.is_none());
    assert!(session_result.is_none());
}

#[tokio::test]
async fn test_transaction_rollback_discards_writes() {
    let db = Sqlite::new_in_memory().unwrap();

    let mut tx = db.begin_transaction().unwrap();

    let project = create_test_project("Rollback Project", "/rollback/dir");
    let created_project =
        <Sqlite as Database>::create_project(&db, project, Some(&mut tx)).unwrap();

    let session = create_test_session(created_project.id, "Rollback Session", true);
    let created_session =
        <Sqlite as Database>::create_session(&db, session, Some(&mut tx)).unwrap();

    tx.rollback().unwrap();
    drop(tx);

    let project_result = db.get_project(created_project.id).unwrap();
    let session_result = db.get_session(created_session.id).unwrap();
    assert!(project_result.is_none());
    assert!(session_result.is_none());
}

#[tokio::test]
async fn test_transaction_drop_without_finish_rolls_back() {
    let db = Sqlite::new_in_memory().unwrap();

    let project_id = {
        let mut tx = db.begin_transaction().unwrap();
        let project = create_test_project("Drop Rollback", "/drop/rollback");
        let created = <Sqlite as Database>::create_project(&db, project, Some(&mut tx)).unwrap();
        created.id
    };

    let project_result = db.get_project(project_id).unwrap();
    assert!(project_result.is_none());
}

#[tokio::test]
async fn test_transaction_finish_methods_are_idempotent() {
    let db = Sqlite::new_in_memory().unwrap();

    let mut tx = db.begin_transaction().unwrap();
    tx.commit().unwrap();
    tx.commit().unwrap();
    drop(tx);

    let mut tx = db.begin_transaction().unwrap();
    tx.rollback().unwrap();
    tx.rollback().unwrap();
}

#[tokio::test]
async fn test_project_with_special_characters_in_name() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project(
        "Test \"Project\" with 'quotes' and \\backslash",
        "/test/dir",
    );
    let created = db.create_project(project.clone()).unwrap();

    let retrieved = db.get_project(created.id).unwrap().unwrap();
    assert_eq!(retrieved.name, project.name);
}

#[tokio::test]
async fn test_project_with_unicode_name() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("テストプロジェクト 🚀 Ñoño", "/test/dir");
    let created = db.create_project(project.clone()).unwrap();

    let retrieved = db.get_project(created.id).unwrap().unwrap();
    assert_eq!(retrieved.name, project.name);
}

#[tokio::test]
async fn test_multiple_sequential_operations() {
    let db = Sqlite::new_in_memory().unwrap();

    for i in 0..10 {
        let project = create_test_project(&format!("Project {}", i), &format!("/dir/{}", i));
        db.create_project(project).unwrap();
    }

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 10);
}

#[tokio::test]
async fn test_update_project_preserves_created_at() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created = db.create_project(project).unwrap();
    let original_created_at = created.created_at;

    let mut updated = created.clone();
    updated.name = "Updated".to_string();
    let result = db.update_project(updated).unwrap();

    assert_eq!(result.created_at, original_created_at);
    assert!(result.updated_at > original_created_at);
}

#[tokio::test]
async fn test_update_session_preserves_created_at() {
    let db = Sqlite::new_in_memory().unwrap();
    let project = create_test_project("Test", "/test/dir");
    let created_project = db.create_project(project).unwrap();

    let session = create_test_session(created_project.id, "Test", true);
    let created = db.create_session(session).unwrap();
    let original_created_at = created.created_at;

    let mut updated = created.clone();
    updated.name = "Updated".to_string();
    let result = db.update_session(updated).unwrap();

    assert_eq!(result.created_at, original_created_at);
    assert!(result.updated_at > original_created_at);
}

#[tokio::test]
async fn test_create_project_with_very_long_name() {
    let db = Sqlite::new_in_memory().unwrap();
    let long_name = "a".repeat(1000);
    let project = create_test_project(&long_name, "/test/dir");

    let created = db.create_project(project.clone()).unwrap();
    assert_eq!(created.name, long_name);
}

#[tokio::test]
async fn test_session_isolation_between_projects() {
    let db = Sqlite::new_in_memory().unwrap();
    let project1 = create_test_project("Project 1", "/dir1");
    let project2 = create_test_project("Project 2", "/dir2");
    let created_project1 = db.create_project(project1).unwrap();
    let created_project2 = db.create_project(project2).unwrap();

    let session1 = create_test_session(created_project1.id, "Session 1", true);
    let session2 = create_test_session(created_project2.id, "Session 2", true);
    let created_session1 = db.create_session(session1).unwrap();
    let created_session2 = db.create_session(session2).unwrap();

    let sessions1 = db.list_sessions_by_project(created_project1.id).unwrap();
    assert_eq!(sessions1.len(), 1);
    assert_eq!(sessions1[0].id, created_session1.id);

    let sessions2 = db.list_sessions_by_project(created_project2.id).unwrap();
    assert_eq!(sessions2.len(), 1);
    assert_eq!(sessions2[0].id, created_session2.id);
}
