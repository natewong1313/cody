use tonic::{Request, transport::Channel};

use crate::backend::{Session, SessionClient, proto_session::CreateSessionRequest};

pub fn create_session(backend_channel: Channel, session: Session) {
    tokio::spawn(async move {
        log::debug!("creating session");
        let mut client = SessionClient::new(backend_channel);
        let request = CreateSessionRequest {
            session: Some(session.into()),
        };

        log::debug!("sending create session request");
        if let Err(error) = client.create_session(Request::new(request)).await {
            log::error!("failed to create session via gRPC: {error}");
        }
    });
}
