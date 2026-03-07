use std::process::Stdio;

use agent_client_protocol::{self as acp, ClientCapabilities, InitializeRequest, ProtocolVersion, Agent as _};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

mod opencode;

pub struct DummyAgentClient {}
#[async_trait::async_trait(?Send)]
impl acp::Client for DummyAgentClient {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn write_text_file(
        &self,
        _args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn read_text_file(
        &self,
        _args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn create_terminal(
        &self,
        _args: acp::CreateTerminalRequest,
    ) -> Result<acp::CreateTerminalResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn terminal_output(
        &self,
        _args: acp::TerminalOutputRequest,
    ) -> acp::Result<acp::TerminalOutputResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn release_terminal(
        &self,
        _args: acp::ReleaseTerminalRequest,
    ) -> acp::Result<acp::ReleaseTerminalResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: acp::WaitForTerminalExitRequest,
    ) -> acp::Result<acp::WaitForTerminalExitResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<(), acp::Error> {
        match args.update {
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) => {
                let text = match content {
                    acp::ContentBlock::Text(text_content) => text_content.text,
                    acp::ContentBlock::Image(_) => "<image>".into(),
                    acp::ContentBlock::Audio(_) => "<audio>".into(),
                    acp::ContentBlock::ResourceLink(resource_link) => resource_link.uri,
                    acp::ContentBlock::Resource(_) => "<resource>".into(),
                    _ => "unknown".into(),
                };
                println!("| Agent: {text}");
            }
            _ => {} // Handle future variants gracefully
        }
        Ok(())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Err(acp::Error::method_not_found())
    }
}

pub struct AgentProcess {
    proc: tokio::process::Child,
    outgoing: Compat<tokio::process::ChildStdin>,
    incoming: Compat<tokio::process::ChildStdout>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Agent {
    Opencode,
}

pub struct AgentHub {
    // agents: HashMap<Agent, AgentProcess>,
}

#[derive(thiserror::Error, Debug)]
pub enum AgentHubError {
    #[error("Failed to spawn opencode process")]
    Spawn(#[from] std::io::Error),
    #[error("Failed to get stdin")]
    GetStdin,
    #[error("Failed to get stdout")]
    GetStdout,
}

#[derive(thiserror::Error, Debug)]
pub enum AgentHubSessionError {
    #[error("Failed to get stdin")]
    AgentUnavailable,
}

impl AgentHub {
    pub async fn new() -> Result<Self, AgentHubError> {
        // let mut agents = HashMap::new();
        // let opencode_process = AgentHub::spawn_agent_from_cmd("opencode", &["acp"]).await?;
        // agents.insert(Agent::Opencode, opencode_process);

        Ok(Self { 
            // agents
        })
    }

    /// Spawns an agent CLI via a child process and tracks stdin and stdout
    async fn spawn_agent_from_cmd(
        command: &str,
        args: &[&str],
    ) -> Result<AgentProcess, AgentHubError> {
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let outgoing = child
            .stdin
            .take()
            .ok_or(AgentHubError::GetStdin)?
            .compat_write();
        let incoming = child
            .stdout
            .take()
            .ok_or(AgentHubError::GetStdout)?
            .compat();

        Ok(AgentProcess {
            proc: child,
            outgoing,
            incoming,
        })
    }

    // pub fn list_agents(&self) -> Vec<Agent> {
    //     self.agents.keys().cloned().collect()
    // }

    pub async fn new_session(&mut self) -> Result<(), AgentHubSessionError> {
        // let agent_process = match self.agents.get(&agent){
        //     Some(proc) => proc,
        //     None => {
        //         return Err(AgentHubSessionError::AgentUnavailable)
        //     }
        // };
        // For now, we'll spawn a process per session. in the future, we will optimize this

        let mut opencode_process = AgentHub::spawn_agent_from_cmd("opencode", &["acp"]).await.unwrap();

        let local_set = tokio::task::LocalSet::new();
        local_set.run_until(async move {

            let (conn, handle_io) = acp::ClientSideConnection::new(
                DummyAgentClient {},
                opencode_process.outgoing,
                opencode_process.incoming,
                |fut| {
                    tokio::task::spawn_local(fut);
                },
            );
            tokio::task::spawn_local(handle_io);

            let init_request = InitializeRequest::new(ProtocolVersion::LATEST).client_capabilities(ClientCapabilities::default());
            conn.initialize(init_request).await.unwrap();

        }).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_client_spawn() {
        let hub = AgentHub::new().await.unwrap();
        let _ = hub;
        // let agents = hub.list_agents();
        // assert_eq!(agents.len(), 1);
        // assert_eq!(agents.get(0), Some(&Agent::Opencode));
    }

    #[tokio::test]
    async fn test_session() {
        let mut hub = AgentHub::new().await.unwrap();
        hub.new_session().await.unwrap()
        // hub.
    }
}
