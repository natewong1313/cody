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
        let json_body = serde_json::to_string_pretty(&request)
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
}
