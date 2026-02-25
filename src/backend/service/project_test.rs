use futures::StreamExt;
use tokio::time::{Duration, timeout};
use tonic::{Code, Request};
use uuid::Uuid;

use crate::backend::{
    proto_project::{
        CreateProjectRequest, DeleteProjectRequest, GetProjectRequest, ListProjectsRequest,
        SubscribeProjectRequest, SubscribeProjectsRequest, UpdateProjectRequest,
        project_server::Project as ProjectService,
    },
    service::test_helpers::{closed_port, test_backend, test_project, valid_project_model},
};

#[tokio::test]
async fn list_projects_returns_models() {
    let backend = test_backend(closed_port());
    let seeded = test_project("proj", "/tmp/proj");
    backend
        .project_repo
        .create(&seeded)
        .await
        .expect("seed create should succeed");

    let response = backend
        .list_projects(Request::new(ListProjectsRequest {}))
        .await
        .expect("list_projects should succeed");
    let projects = response.into_inner().projects;

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, seeded.id.to_string());
    assert_eq!(projects[0].name, "proj");
    assert_eq!(projects[0].dir, "/tmp/proj");
}

#[tokio::test]
async fn get_project_returns_none_for_missing() {
    let backend = test_backend(closed_port());

    let response = backend
        .get_project(Request::new(GetProjectRequest {
            project_id: Uuid::new_v4().to_string(),
        }))
        .await
        .expect("get_project should succeed");

    assert!(response.into_inner().project.is_none());
}

#[tokio::test]
async fn get_project_rejects_invalid_uuid() {
    let backend = test_backend(closed_port());

    let err = backend
        .get_project(Request::new(GetProjectRequest {
            project_id: "not-a-uuid".to_string(),
        }))
        .await
        .expect_err("invalid uuid should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("project_id"));
}

#[tokio::test]
async fn create_project_rejects_missing_project_field() {
    let backend = test_backend(closed_port());

    let err = backend
        .create_project(Request::new(CreateProjectRequest { project: None }))
        .await
        .expect_err("missing project should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("missing project"));
}

#[tokio::test]
async fn create_project_rejects_invalid_model() {
    let backend = test_backend(closed_port());
    let mut invalid = valid_project_model();
    invalid.id = "bad-uuid".to_string();

    let err = backend
        .create_project(Request::new(CreateProjectRequest {
            project: Some(invalid),
        }))
        .await
        .expect_err("invalid model should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("project.id"));
}

#[tokio::test]
async fn update_project_rejects_invalid_model() {
    let backend = test_backend(closed_port());
    let mut invalid = valid_project_model();
    invalid.created_at = "not-a-datetime".to_string();

    let err = backend
        .update_project(Request::new(UpdateProjectRequest {
            project: Some(invalid),
        }))
        .await
        .expect_err("invalid model should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("project.created_at"));
}

#[tokio::test]
async fn create_and_delete_project_happy_path() {
    let backend = test_backend(closed_port());
    let model = valid_project_model();

    let created = backend
        .create_project(Request::new(CreateProjectRequest {
            project: Some(model.clone()),
        }))
        .await
        .expect("create_project should succeed")
        .into_inner()
        .project
        .expect("created project should exist");

    backend
        .delete_project(Request::new(DeleteProjectRequest {
            project_id: created.id.clone(),
        }))
        .await
        .expect("delete_project should succeed");

    let fetched = backend
        .get_project(Request::new(GetProjectRequest {
            project_id: created.id,
        }))
        .await
        .expect("get_project should succeed")
        .into_inner()
        .project;

    assert!(fetched.is_none());
}

#[tokio::test]
async fn subscribe_projects_returns_snapshot_once() {
    let backend = test_backend(closed_port());
    let seeded = test_project("proj", "/tmp/proj");
    backend
        .project_repo
        .create(&seeded)
        .await
        .expect("seed create should succeed");

    let response = backend
        .subscribe_projects(Request::new(SubscribeProjectsRequest {}))
        .await
        .expect("subscribe_projects should succeed");

    let mut stream = response.into_inner();

    let first = stream
        .next()
        .await
        .expect("stream should yield first item")
        .expect("first item should be ok");

    assert_eq!(first.projects.len(), 1);
    assert_eq!(first.projects[0].id, seeded.id.to_string());
    assert_eq!(first.projects[0].name, "proj");
    assert_eq!(first.projects[0].dir, "/tmp/proj");

    let next_item = timeout(Duration::from_millis(50), stream.next()).await;
    assert!(
        next_item.is_err(),
        "stream should stay open without emitting updates"
    );
}

#[tokio::test]
async fn subscribe_project_returns_current_snapshot_for_existing_project() {
    let backend = test_backend(closed_port());
    let seeded = test_project("proj", "/tmp/proj");
    backend
        .project_repo
        .create(&seeded)
        .await
        .expect("seed create should succeed");

    let response = backend
        .subscribe_project(Request::new(SubscribeProjectRequest {
            project_id: seeded.id.to_string(),
        }))
        .await
        .expect("subscribe_project should succeed");

    let mut stream = response.into_inner();
    let first = stream
        .next()
        .await
        .expect("stream should yield first item")
        .expect("first item should be ok");

    let project = first.project.expect("project should be present");
    assert_eq!(project.id, seeded.id.to_string());
    assert_eq!(project.name, seeded.name);
    assert_eq!(project.dir, seeded.dir);
}

#[tokio::test]
async fn subscribe_project_emits_update_and_delete_events() {
    let backend = test_backend(closed_port());
    let seeded = test_project("proj", "/tmp/proj");
    let created = backend
        .project_repo
        .create(&seeded)
        .await
        .expect("seed create should succeed");

    let response = backend
        .subscribe_project(Request::new(SubscribeProjectRequest {
            project_id: created.id.to_string(),
        }))
        .await
        .expect("subscribe_project should succeed");
    let mut stream = response.into_inner();

    let _initial = stream
        .next()
        .await
        .expect("stream should yield initial item")
        .expect("initial item should be ok");

    let mut updated_model = valid_project_model();
    updated_model.id = created.id.to_string();
    updated_model.name = "updated".to_string();
    updated_model.dir = "/tmp/updated".to_string();
    updated_model.created_at = created
        .created_at
        .format("%Y-%m-%d %H:%M:%S%.f")
        .to_string();

    backend
        .update_project(Request::new(UpdateProjectRequest {
            project: Some(updated_model),
        }))
        .await
        .expect("update_project should succeed");

    let update = timeout(Duration::from_millis(200), stream.next())
        .await
        .expect("stream should emit update")
        .expect("stream should yield item")
        .expect("update item should be ok");
    let update_project = update.project.expect("updated project should be present");
    assert_eq!(update_project.id, created.id.to_string());
    assert_eq!(update_project.name, "updated");
    assert_eq!(update_project.dir, "/tmp/updated");

    backend
        .delete_project(Request::new(DeleteProjectRequest {
            project_id: created.id.to_string(),
        }))
        .await
        .expect("delete_project should succeed");

    let deleted = timeout(Duration::from_millis(200), stream.next())
        .await
        .expect("stream should emit delete")
        .expect("stream should yield item")
        .expect("delete item should be ok");
    assert!(
        deleted.project.is_none(),
        "deleted project event should clear project details"
    );
}
