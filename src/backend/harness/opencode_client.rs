#![allow(dead_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

#[derive(Clone)]
pub struct OpencodeApiClient {
    http_client: Client,
    server_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeSession {
    pub id: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    #[serde(rename = "providerID", alias = "providerId")]
    pub provider_id: String,
    #[serde(rename = "modelID", alias = "modelId")]
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProviderModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProviderInfo {
    pub id: String,
    pub name: String,
    pub models: HashMap<String, OpencodeProviderModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProviderListResponse {
    pub all: Vec<OpencodeProviderInfo>,
    pub default: HashMap<String, String>,
    pub connected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum OpencodePartInput {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        synthetic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ignored: Option<bool>,
    },
    OpencodeFile {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        mime: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        url: String,
    },
    Agent {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<OpencodeSourceRange>,
    },
    Subtask {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        prompt: String,
        description: String,
        agent: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeSourceRange {
    pub value: String,
    pub start: i32,
    pub end: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeSendMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_reply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,
    pub parts: Vec<OpencodePartInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeCreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "parentID", alias = "parentId")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<serde_json::Value>,
}

// OpencodeMessage types from types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageTime {
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageTimeCompleted {
    pub created: i64,
    pub completed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub file: String,
    pub before: String,
    pub after: String,
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSummary {
    pub title: Option<String>,
    pub body: Option<String>,
    pub diffs: Vec<FileDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    #[serde(rename = "providerID", alias = "providerId")]
    pub provider_id: String,
    #[serde(rename = "modelID", alias = "modelId")]
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePath {
    pub cwd: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub input: i32,
    pub output: i32,
    pub reasoning: i32,
    pub cache: CacheInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInfo {
    pub read: i32,
    pub write: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAuthError {
    pub name: String,
    pub data: ProviderAuthErrorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAuthErrorData {
    pub provider_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownError {
    pub name: String,
    pub data: UnknownErrorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownErrorData {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub name: String,
    pub data: ApiErrorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorData {
    pub message: String,
    pub status_code: Option<i32>,
    pub is_retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "role")]
pub enum OpencodeMessage {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
}

impl OpencodeMessage {
    pub fn id(&self) -> &str {
        match self {
            OpencodeMessage::User(u) => &u.id,
            OpencodeMessage::Assistant(a) => &a.id,
        }
    }

    pub fn session_id(&self) -> &str {
        match self {
            OpencodeMessage::User(u) => &u.session_id,
            OpencodeMessage::Assistant(a) => &a.session_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub time: MessageTime,
    pub summary: Option<MessageSummary>,
    pub agent: String,
    pub model: ModelInfo,
    pub system: Option<String>,
    pub tools: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    pub time: MessageTimeCompleted,
    pub error: Option<OpencodeMessageError>,
    #[serde(rename = "parentID", alias = "parentId")]
    pub parent_id: String,
    #[serde(rename = "modelID", alias = "modelId")]
    pub model_id: String,
    #[serde(rename = "providerID", alias = "providerId")]
    pub provider_id: String,
    pub mode: String,
    pub path: MessagePath,
    pub cost: f64,
    pub tokens: TokenInfo,
    pub finish: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum OpencodeMessageError {
    ProviderAuth(ProviderAuthError),
    Unknown(UnknownError),
    Api(ApiError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeTextPart {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    pub text: String,
    pub synthetic: Option<bool>,
    pub ignored: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeReasoningPart {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeToolStateCompleted {
    pub status: String,
    pub input: serde_json::Value,
    pub output: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeToolPart {
    pub id: String,
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
    #[serde(rename = "callID", alias = "callId")]
    pub call_id: String,
    pub tool: String,
    pub state: OpencodeToolState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum OpencodeToolState {
    Pending(OpencodeToolStatePending),
    Running(OpencodeToolStateRunning),
    Completed(OpencodeToolStateCompleted),
    Error(OpencodeToolStateError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeToolStatePending {
    pub status: String,
    pub input: serde_json::Value,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeToolStateRunning {
    pub status: String,
    pub input: serde_json::Value,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeToolStateError {
    pub status: String,
    pub input: serde_json::Value,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum OpencodePart {
    #[serde(rename = "text")]
    Text(OpencodeTextPart),
    #[serde(rename = "reasoning")]
    Reasoning(OpencodeReasoningPart),
    #[serde(rename = "tool")]
    Tool(OpencodeToolPart),
}

impl OpencodePart {
    pub fn session_id(&self) -> &str {
        match self {
            OpencodePart::Text(t) => &t.session_id,
            OpencodePart::Reasoning(r) => &r.session_id,
            OpencodePart::Tool(t) => &t.session_id,
        }
    }

    pub fn message_id(&self) -> &str {
        match self {
            OpencodePart::Text(t) => &t.message_id,
            OpencodePart::Reasoning(r) => &r.message_id,
            OpencodePart::Tool(t) => &t.message_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeMessageWithParts {
    pub info: OpencodeMessage,
    pub parts: Vec<OpencodePart>,
}

impl OpencodeMessageWithParts {
    pub fn id(&self) -> &str {
        self.info.id()
    }

    pub fn session_id(&self) -> &str {
        self.info.session_id()
    }
}

// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeGlobalEvent {
    pub directory: Option<String>,
    pub payload: OpencodeEventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum OpencodeEventPayload {
    #[serde(rename = "message.updated")]
    MessageUpdated {
        #[serde(rename = "properties")]
        props: OpencodeMessageUpdatedProps,
    },
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated {
        #[serde(rename = "properties")]
        props: OpencodeMessagePartUpdatedProps,
    },
    #[serde(rename = "message.removed")]
    MessageRemoved {
        #[serde(rename = "properties")]
        props: OpencodeMessageRemovedProps,
    },
    #[serde(rename = "session.idle")]
    SessionIdle {
        #[serde(rename = "properties")]
        props: OpencodeSessionIdleProps,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeMessageUpdatedProps {
    pub info: OpencodeMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeMessagePartUpdatedProps {
    pub part: OpencodePart,
    pub delta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeMessageRemovedProps {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
    #[serde(rename = "messageID", alias = "messageId")]
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeSessionIdleProps {
    #[serde(rename = "sessionID", alias = "sessionId")]
    pub session_id: String,
}

impl OpencodeApiClient {
    pub fn new(port: u32) -> Self {
        Self {
            http_client: Client::new(),
            server_url: format!("http://127.0.0.1:{port}"),
        }
    }

    pub async fn get_sessions(&self) -> anyhow::Result<Vec<OpencodeSession>> {
        let sessions: Vec<OpencodeSession> = self
            .http_client
            .get(format!("{}/session", self.server_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(sessions)
    }

    pub async fn create_session(
        &self,
        request: Option<&OpencodeCreateSessionRequest>,
        directory: Option<&str>,
    ) -> anyhow::Result<OpencodeSession> {
        let mut req = self
            .http_client
            .post(format!("{}/session", self.server_url));

        if let Some(dir) = directory {
            req = req.query(&[("directory", dir)]);
        }
        if let Some(body) = request {
            req = req.json(body);
        }

        let session: OpencodeSession = req.send().await?.json().await?;
        Ok(session)
    }

    pub async fn get_event_stream(
        &self,
    ) -> Result<
        Pin<
            Box<
                dyn futures::Stream<
                        Item = Result<
                            eventsource_stream::Event,
                            eventsource_stream::EventStreamError<reqwest::Error>,
                        >,
                    > + Send,
            >,
        >,
        reqwest::Error,
    > {
        use eventsource_stream::Eventsource;

        let response = self
            .http_client
            .get(format!("{}/event", self.server_url))
            .send()
            .await?;

        Ok(Box::pin(response.bytes_stream().eventsource()))
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        request: &OpencodeSendMessageRequest,
        directory: Option<&str>,
    ) -> anyhow::Result<OpencodeMessageWithParts> {
        let mut req = self
            .http_client
            .post(format!(
                "{}/session/{}/message",
                self.server_url, session_id
            ))
            .json(request);
        if let Some(dir) = directory {
            req = req.query(&[("directory", dir)]);
        }
        let response = req.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "opencode send_message failed with status {status}: {body}"
            ));
        }

        serde_json::from_str::<OpencodeMessageWithParts>(&body).map_err(|err| {
            anyhow::anyhow!(
                "opencode send_message returned unexpected body: {err}; status={status}; body={body}"
            )
        })
    }

    pub async fn get_session_messages(
        &self,
        session_id: &str,
        limit: Option<i32>,
        directory: Option<&str>,
    ) -> anyhow::Result<Vec<OpencodeMessageWithParts>> {
        let mut request = self.http_client.get(format!(
            "{}/session/{}/message",
            self.server_url, session_id
        ));
        if let Some(l) = limit {
            request = request.query(&[("limit", l.to_string())]);
        }
        if let Some(dir) = directory {
            request = request.query(&[("directory", dir)]);
        }
        let messages: Vec<OpencodeMessageWithParts> = request.send().await?.json().await?;
        Ok(messages)
    }

    pub async fn get_providers(
        &self,
        directory: Option<&str>,
    ) -> anyhow::Result<OpencodeProviderListResponse> {
        let mut request = self
            .http_client
            .get(format!("{}/provider", self.server_url));
        if let Some(dir) = directory {
            request = request.query(&[("directory", dir)]);
        }
        let response: OpencodeProviderListResponse = request.send().await?.json().await?;
        Ok(response)
    }
}
