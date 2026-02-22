use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::backend::harness::Harness;
use crate::backend::{
    data::session::Session,
    harness::opencode_client::{OpencodeApiClient, OpencodeCreateSessionRequest},
};

#[derive(Error, Debug)]
pub enum OpencodeHarnessError {
    #[error("Failed to spawn opencode process: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("Mutex poisoned")]
    MutexPoisoned,

    #[error("API request failed: {0}")]
    ApiRequest(String),
}

// Needs to be clonable since we pass this around in the repos
#[derive(Clone)]
pub struct OpencodeHarness {
    proc: Arc<Mutex<Option<Child>>>,
    opencode_client: OpencodeApiClient,
}

impl Harness for OpencodeHarness {
    fn new() -> anyhow::Result<Self> {
        let port = 6767;
        let proc = unsafe {
            Command::new("opencode")
                .arg("serve")
                .arg("--port")
                .arg("6767")
                .arg("--print-logs")
                .arg("--log-level")
                .arg("DEBUG")
                .pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                })
                .spawn()
                .map_err(|e| anyhow::anyhow!(OpencodeHarnessError::Spawn(e)))?
        };

        let opencode_client = OpencodeApiClient::new(port);

        Ok(Self {
            proc: Arc::new(Mutex::new(Some(proc))),
            opencode_client,
        })
    }

    fn cleanup(&self) {
        let Ok(mut proc) = self.proc.lock() else {
            return;
        };

        let Some(proc) = proc.take() else {
            return;
        };

        let pid = proc.id() as i32;
        unsafe {
            libc::kill(-pid, libc::SIGTERM);
        }
    }

    async fn create_session(
        &self,
        session: Session,
        directory: Option<&str>,
    ) -> anyhow::Result<()> {
        let request = OpencodeCreateSessionRequest {
            parent_id: None,
            title: Some(session.name),
            permission: None,
        };

        self.opencode_client
            .create_session(Some(&request), directory)
            .await
            .map_err(|e| anyhow::anyhow!(OpencodeHarnessError::ApiRequest(e.to_string())))?;

        Ok(())
    }
}

#[cfg(test)]
impl OpencodeHarness {
    pub(crate) fn new_for_test(port: u32) -> Self {
        Self {
            proc: Arc::new(Mutex::new(None)),
            opencode_client: OpencodeApiClient::new(port),
        }
    }
}

impl Drop for OpencodeHarness {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_creation_flow() {
//         let harness = OpencodeHarness::new();
//         assert_eq!(harness.is_ok(), true);
//         assert_eq!(harness.unwrap().cleanup().is_ok(), true);
//     }
// }
