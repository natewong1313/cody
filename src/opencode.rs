use libc::SIGTERM;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

pub struct OpencodeProcess {
    proc_handle: Child,
}

impl OpencodeProcess {
    pub fn start(port: u32) -> anyhow::Result<Self> {
        let proc = Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--print-logs")
            .arg("--log-level")
            .arg("DEBUG")
            .process_group(0)
            .spawn()?;

        Ok(Self { proc_handle: proc })
    }

    pub fn stop(&self) {
        let pid = self.proc_handle.id() as i32;
        unsafe {
            libc::kill(-pid, SIGTERM);
        }
    }
}

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
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PartInput {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        synthetic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ignored: Option<bool>,
    },
    File {
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
        source: Option<SourceRange>,
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
pub struct SourceRange {
    pub value: String,
    pub start: i32,
    pub end: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub parts: Vec<PartInput>,
}

// Message types from types.ts
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
    pub provider_id: String,
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
pub enum Message {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
}

impl Message {
    pub fn id(&self) -> &str {
        match self {
            Message::User(u) => &u.id,
            Message::Assistant(a) => &a.id,
        }
    }

    pub fn session_id(&self) -> &str {
        match self {
            Message::User(u) => &u.session_id,
            Message::Assistant(a) => &a.session_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
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
    pub session_id: String,
    pub role: String,
    pub time: MessageTimeCompleted,
    pub error: Option<MessageError>,
    pub parent_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub mode: String,
    pub path: MessagePath,
    pub cost: f64,
    pub tokens: TokenInfo,
    pub finish: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum MessageError {
    ProviderAuth(ProviderAuthError),
    Unknown(UnknownError),
    Api(ApiError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPart {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: String,
    pub synthetic: Option<bool>,
    pub ignored: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningPart {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStateCompleted {
    pub status: String,
    pub input: serde_json::Value,
    pub output: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPart {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    pub call_id: String,
    pub tool: String,
    pub state: ToolState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ToolState {
    Pending(ToolStatePending),
    Running(ToolStateRunning),
    Completed(ToolStateCompleted),
    Error(ToolStateError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatePending {
    pub status: String,
    pub input: serde_json::Value,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStateRunning {
    pub status: String,
    pub input: serde_json::Value,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStateError {
    pub status: String,
    pub input: serde_json::Value,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Part {
    #[serde(rename = "text")]
    Text(TextPart),
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningPart),
    #[serde(rename = "tool")]
    Tool(ToolPart),
}

impl Part {
    pub fn session_id(&self) -> &str {
        match self {
            Part::Text(t) => &t.session_id,
            Part::Reasoning(r) => &r.session_id,
            Part::Tool(t) => &t.session_id,
        }
    }

    pub fn message_id(&self) -> &str {
        match self {
            Part::Text(t) => &t.message_id,
            Part::Reasoning(r) => &r.message_id,
            Part::Tool(t) => &t.message_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageWithParts {
    pub info: Message,
    pub parts: Vec<Part>,
}

impl MessageWithParts {
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
pub struct GlobalEvent {
    pub directory: String,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum EventPayload {
    #[serde(rename = "message.updated")]
    MessageUpdated {
        #[serde(rename = "properties")]
        props: MessageUpdatedProps,
    },
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated {
        #[serde(rename = "properties")]
        props: MessagePartUpdatedProps,
    },
    #[serde(rename = "message.removed")]
    MessageRemoved {
        #[serde(rename = "properties")]
        props: MessageRemovedProps,
    },
    #[serde(rename = "session.idle")]
    SessionIdle {
        #[serde(rename = "properties")]
        props: SessionIdleProps,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdatedProps {
    pub info: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartUpdatedProps {
    pub part: Part,
    pub delta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRemovedProps {
    pub session_id: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIdleProps {
    pub session_id: String,
}

impl OpencodeApiClient {
    pub fn new(port: u32) -> Self {
        Self {
            http_client: Client::new(),
            server_url: format!("http://127.0.0.1:{}", port),
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

    pub async fn create_session(&self) -> anyhow::Result<OpencodeSession> {
        let session: OpencodeSession = self
            .http_client
            .post(format!("{}/session", self.server_url))
            .json(&serde_json::json!({}))
            .send()
            .await?
            .json()
            .await?;
        Ok(session)
    }

    pub async fn get_event_stream(
        &self,
    ) -> Result<
        impl futures::Stream<
            Item = Result<
                eventsource_stream::Event,
                eventsource_stream::EventStreamError<reqwest::Error>,
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

        Ok(response.bytes_stream().eventsource())
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        request: SendMessageRequest,
    ) -> Result<(), reqwest::Error> {
        let _json_body = serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize".to_string());

        self.http_client
            .post(format!(
                "{}/session/{}/message",
                self.server_url, session_id
            ))
            .json(&request)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_session_messages(
        &self,
        session_id: &str,
    ) -> anyhow::Result<Vec<MessageWithParts>> {
        let messages: Vec<MessageWithParts> = self
            .http_client
            .get(format!("{}/session/{}/message", self.server_url, session_id))
            .send()
            .await?
            .json()
            .await?;
        Ok(messages)
    }
}
