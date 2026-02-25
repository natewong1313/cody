use egui::Ui;
use egui_inbox::UiInbox;
use futures::StreamExt;
use std::collections::{HashMap, HashSet};
use tonic::{Request, transport::Channel};
use uuid::Uuid;

use crate::backend::proto_message::{
    MessageModel, SubscribeSessionMessagesReply, SubscribeSessionMessagesRequest,
    message_client::MessageClient,
};

use super::QueryState;

pub type MessagesState = QueryState<Vec<MessageModel>>;

pub struct Messages {
    backend_channel: Channel,
    state_by_session: HashMap<Uuid, MessagesState>,
    session_subscriptions: HashSet<Uuid>,
    inbox: UiInbox<(Uuid, MessagesState)>,
}

impl Messages {
    pub fn new(backend_channel: Channel) -> Self {
        Self {
            backend_channel,
            state_by_session: HashMap::new(),
            session_subscriptions: HashSet::new(),
            inbox: UiInbox::new(),
        }
    }

    pub fn subscribe_state(&mut self, ui: &Ui, session_id: Uuid) -> MessagesState {
        for (updated_session_id, updated_state) in self.inbox.read(ui) {
            self.state_by_session
                .insert(updated_session_id, updated_state);
        }

        self.subscribe_session_if_needed(session_id);

        self.state_by_session
            .get(&session_id)
            .cloned()
            .unwrap_or(QueryState::Loading)
    }

    fn subscribe_session_if_needed(&mut self, session_id: Uuid) {
        if self.session_subscriptions.contains(&session_id) {
            return;
        }

        self.session_subscriptions.insert(session_id);
        self.state_by_session
            .insert(session_id, QueryState::Loading);

        let sender = self.inbox.sender().clone();
        let channel = self.backend_channel.clone();

        tokio::spawn(async move {
            let mut stream = match MessageClient::new(channel)
                .subscribe_session_messages(Request::new(SubscribeSessionMessagesRequest {
                    session_id: session_id.to_string(),
                }))
                .await
            {
                Ok(resp) => resp.into_inner(),
                Err(e) => {
                    let _ = sender.send((session_id, QueryState::Error(e.to_string())));
                    return;
                }
            };

            while let Some(next) = stream.next().await {
                match next.map_err(|e| e.to_string()).and_then(Messages::map) {
                    Ok(messages) => {
                        let _ = sender.send((session_id, QueryState::Data(messages)));
                    }
                    Err(e) => {
                        let _ = sender.send((session_id, QueryState::Error(e)));
                        return;
                    }
                }
            }

            let _ = sender.send((
                session_id,
                QueryState::Error("messages stream closed unexpectedly".to_string()),
            ));
        });
    }

    fn map(reply: SubscribeSessionMessagesReply) -> Result<Vec<MessageModel>, String> {
        Ok(reply.messages)
    }
}
