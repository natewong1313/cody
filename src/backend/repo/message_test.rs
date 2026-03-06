use chrono::{Duration, NaiveDateTime};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::Database,
    harness::Harness,
    harness::opencode::OpencodeHarness,
    models::project_model::ProjectModel,
    models::session_model::SessionModel,
    repo::{
        assistant_message::AssistantMessage,
        message::{Message, MessageRepo, MessageRepoError},
        user_message::UserMessage,
        user_message_part::UserMessagePart,
    },
};

use super::test_utils::{closed_port, fixed_datetime, wait_for_port};

fn test_project(id: Uuid, at: NaiveDateTime) -> ProjectModel {
    ProjectModel {
        id,
        name: "proj".to_string(),
        dir: "/tmp/proj".to_string(),
        created_at: at,
        updated_at: at,
    }
}

fn test_session(id: Uuid, project_id: Uuid, at: NaiveDateTime) -> SessionModel {
    SessionModel {
        id,
        project_id,
        parent_session_id: None,
        show_in_gui: true,
        name: "sess".to_string(),
        harness_type: "opencode".to_string(),
        harness_session_id: format!("hs-{id}"),
        dir: Some("/tmp/proj".to_string()),
        summary_additions: None,
        summary_deletions: None,
        summary_files: None,
        created_at: at,
        updated_at: at,
    }
}

fn user_message(id: Uuid, session_id: Uuid, at: NaiveDateTime) -> UserMessage {
    UserMessage {
        id,
        session_id,
        agent: "build".to_string(),
        model_provider_id: "workers-ai".to_string(),
        model_id: "@cf/moonshotai/kimi-k2.5".to_string(),
        system_prompt: None,
        structured_output_type: "text".to_string(),
        tools_list: "{}".to_string(),
        thinking_variant: None,
        created_at: at,
        updated_at: at,
    }
}

fn assistant_message(
    id: Uuid,
    session_id: Uuid,
    user_message_id: Uuid,
    at: NaiveDateTime,
) -> AssistantMessage {
    AssistantMessage {
        id,
        harness_message_id: None,
        session_id,
        user_message_id,
        agent: "build".to_string(),
        model_provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
        cwd: "/tmp/proj".to_string(),
        root: "/tmp/proj".to_string(),
        cost: 0.0,
        token_total: Some(3),
        token_input: 1,
        token_output: 2,
        token_reasoning: 0,
        token_cache_read: 0,
        token_cache_write: 0,
        error_message: None,
        created_at: at,
        updated_at: at,
        completed_at: Some(at),
    }
}

fn user_message_part(
    id: Uuid,
    user_message_id: Uuid,
    session_id: Uuid,
    position: i64,
    text: &str,
    at: NaiveDateTime,
) -> UserMessagePart {
    UserMessagePart {
        id,
        user_message_id,
        session_id,
        position,
        part_type: "text".to_string(),
        text: Some(text.to_string()),
        file_name: None,
        file_url: None,
        agent_name: None,
        subtask_prompt: None,
        subtask_description: None,
        created_at: at,
        updated_at: at,
    }
}

#[tokio::test]
async fn create_user_message_sends_message_to_harness() {
    let port = closed_port();
    let harness = OpencodeHarness::new_with_process_for_test(port)
        .expect("test harness with process should start");
    wait_for_port(port);

    let db = Database::new_in_memory()
        .await
        .expect("in-memory db should initialize");
    let now = fixed_datetime();
    let project_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    let project_dir = std::env::temp_dir().join(format!("cody-opencode-test-{session_id}"));
    std::fs::create_dir_all(&project_dir).expect("project temp directory should be created");
    let project_dir_string = project_dir.to_string_lossy().to_string();

    db.create_project(ProjectModel {
        id: project_id,
        name: "proj".to_string(),
        dir: project_dir_string.clone(),
        created_at: now,
        updated_at: now,
    })
    .await
    .expect("create project should succeed");

    let harness_session_id = harness
        .create_session(
            test_session(session_id, project_id, now),
            Some(&project_dir_string),
        )
        .await
        .expect("create opencode session should succeed");

    let mut session = test_session(session_id, project_id, now);
    session.harness_session_id = harness_session_id.clone();
    session.dir = Some(project_dir_string.clone());
    db.create_session(session)
        .await
        .expect("create session should succeed");

    let harness_for_asserts = harness.clone();
    let ctx = BackendContext::new(db, harness);
    let repo = MessageRepo::new(ctx);

    let created = repo
        .create_user_message(
            user_message(message_id, session_id, now + Duration::seconds(1)),
            vec![user_message_part(
                Uuid::new_v4(),
                message_id,
                session_id,
                0,
                "hello from integration test",
                now + Duration::seconds(1),
            )],
        )
        .await
        .expect("create_user_message should succeed");

    assert_eq!(created.id, message_id);
    assert_eq!(created.session_id, session_id);

    let messages = harness_for_asserts
        .get_session_messages(&harness_session_id, Some(50), Some(&project_dir_string))
        .await
        .expect("get_session_messages should succeed");
    assert!(
        messages
            .iter()
            .all(|m| m.session_id() == harness_session_id),
        "all returned messages should belong to test harness session"
    );
}
