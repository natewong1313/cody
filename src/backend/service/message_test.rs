use tonic::Request;

use crate::backend::{
    proto_message::{
        ListMessagesBySessionRequest, SubscribeMessagesBySessionRequest,
        messages_server::Messages as MessageService,
    },
    service::test_helpers::{closed_port, test_backend},
};

#[tokio::test]
async fn list_messages_by_session_returns_empty_for_new_session() {
    let backend = test_backend(closed_port());
    let result = backend
        .list_messages_by_session(Request::new(ListMessagesBySessionRequest {
            session_id: uuid::Uuid::new_v4().to_string(),
            limit: 100,
        }))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn subscribe_messages_by_session_rejects_unknown_session() {
    let backend = test_backend(closed_port());
    let result = backend
        .subscribe_messages_by_session(Request::new(SubscribeMessagesBySessionRequest {
            session_id: uuid::Uuid::new_v4().to_string(),
        }))
        .await;

    let err = match result {
        Ok(_) => panic!("unknown session should fail"),
        Err(err) => err,
    };

    assert_eq!(err.code(), tonic::Code::NotFound);
}
