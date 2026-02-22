use crate::{
    opencode::{OpencodeApiClient, OpencodePartInput, OpencodeSendMessageRequest, OpencodeSession},
    pages::{PageAction, PagesRouter, Route},
};
use egui_inbox::UiInbox;

pub struct ActionContext<'a> {
    pub pages_router: &'a mut PagesRouter,
    pub session_inbox: &'a UiInbox<Result<OpencodeSession, String>>,
}

pub fn handle_action(ctx: &mut ActionContext<'_>, action: PageAction) {
    match action {
        PageAction::Navigate(page) => handle_navigate(ctx.pages_router, page),
        // PageAction::CreateSession => handle_create_session(ctx.api_client, ctx.session_inbox),
        // PageAction::SendMessage {
        //     session_id,
        //     message,
        //     model,
        // } => handle_send_message(ctx.api_client, session_id, message, model),
    }
}

fn handle_navigate(pages_router: &mut PagesRouter, page: Route) {
    pages_router.navigate(page);
}

fn handle_create_session(
    api_client: &OpencodeApiClient,
    session_inbox: &UiInbox<Result<OpencodeSession, String>>,
) {
    let sender = session_inbox.sender();
    let client = api_client.clone();
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(async move {
        match client.create_session(None, None).await {
            Ok(session) => {
                sender.send(Ok(session)).ok();
            }
            Err(e) => {
                sender.send(Err(e.to_string())).ok();
            }
        }
    });
}

fn handle_send_message(
    api_client: &OpencodeApiClient,
    session_id: String,
    message: String,
    model: Option<crate::opencode::ModelSelection>,
) {
    let client = api_client.clone();
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(async move {
        let request = OpencodeSendMessageRequest {
            message_id: None,
            model,
            agent: None,
            no_reply: None,
            system: None,
            tools: None,
            parts: vec![OpencodePartInput::Text {
                id: None,
                text: message,
                synthetic: None,
                ignored: None,
            }],
        };
        match client.send_message(&session_id, &request, None).await {
            Ok(_) => {
                log::info!("Message sent to session {}", session_id);
            }
            Err(e) => {
                log::error!("Failed to send message to session {}: {}", session_id, e);
            }
        }
    });
}
