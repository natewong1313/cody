use libc::SIGTERM;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
}
