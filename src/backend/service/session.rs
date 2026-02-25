use std::sync::Arc;
use tonic::{Request, Response, Status};

use super::required_field;
use crate::backend::{
    BackendService, Session,
    proto_session::{
        CreateSessionReply, CreateSessionRequest, DeleteSessionReply, DeleteSessionRequest,
        GetSessionMessagesReply, GetSessionMessagesRequest, GetSessionReply, GetSessionRequest,
        ListSessionsByProjectReply, ListSessionsByProjectRequest, SendSessionMessageReply,
        SendSessionMessageRequest, UpdateSessionReply, UpdateSessionRequest,
        session_server::Session as SessionService,
    },
    proto_utils::parse_uuid,
};

#[tonic::async_trait]
impl SessionService for Arc<BackendService> {
    async fn list_sessions_by_project(
        &self,
        request: Request<ListSessionsByProjectRequest>,
    ) -> Result<Response<ListSessionsByProjectReply>, Status> {
        let project_id = parse_uuid("project_id", &request.into_inner().project_id)?;
        let sessions = self.session_repo.list_by_project(&project_id).await?;

        Ok(Response::new(ListSessionsByProjectReply {
            sessions: sessions.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        let session = self.session_repo.get(&session_id).await?;

        Ok(Response::new(GetSessionReply {
            session: session.map(Into::into),
        }))
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        let req = request.into_inner();
        let model = required_field(req.session, "session")?;
        let session = Session::try_from(model)?;

        let created = self.session_repo.create(&session).await?;

        Ok(Response::new(CreateSessionReply {
            session: Some(created.into()),
        }))
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<UpdateSessionReply>, Status> {
        let req = request.into_inner();
        let model = required_field(req.session, "session")?;
        let session = Session::try_from(model)?;

        let updated = self.session_repo.update(&session).await?;

        Ok(Response::new(UpdateSessionReply {
            session: Some(updated.into()),
        }))
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let session_id = parse_uuid("session_id", &request.into_inner().session_id)?;
        self.session_repo.delete(&session_id).await?;

        Ok(Response::new(DeleteSessionReply {}))
    }

    async fn send_session_message(
        &self,
        request: Request<SendSessionMessageRequest>,
    ) -> Result<Response<SendSessionMessageReply>, Status> {
        let req = request.into_inner();
        let session_id = parse_uuid("session_id", &req.session_id)?;
        let input = required_field(req.input, "input")?;

        let message = self.session_repo.send_message(&session_id, input).await?;

        Ok(Response::new(SendSessionMessageReply {
            message: Some(message.into()),
        }))
    }

    async fn get_session_messages(
        &self,
        request: Request<GetSessionMessagesRequest>,
    ) -> Result<Response<GetSessionMessagesReply>, Status> {
        let req = request.into_inner();
        let session_id = parse_uuid("session_id", &req.session_id)?;

        let messages = self
            .session_repo
            .get_messages(&session_id, req.limit)
            .await?;

        Ok(Response::new(GetSessionMessagesReply {
            messages: messages.into_iter().map(Into::into).collect(),
        }))
    }
}
