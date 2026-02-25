use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::backend::harness::Harness;
use crate::backend::{
    harness::opencode_client::{OpencodeApiClient, OpencodeCreateSessionRequest},
    repo::session::Session,
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
        log::debug!("Starting opencode on port {port}");
        let proc = unsafe {
            Command::new("opencode")
                .arg("serve")
                .arg("--port")
                .arg(&port.to_string())
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

        log::debug!("Opencode running on port {port}");

        let opencode_client = OpencodeApiClient::new(port);

        Ok(Self {
            proc: Arc::new(Mutex::new(Some(proc))),
            opencode_client,
        })
    }

    fn cleanup(&self) {
        let Ok(mut proc) = self.proc.lock() else {
            log::error!("Failed to lock process mutex during cleanup");
            return;
        };

        let Some(mut proc) = proc.take() else {
            log::debug!("No opencode process to cleanup");
            return;
        };

        let pid = proc.id() as i32;
        log::debug!("Cleaning up opencode process (PID: {})", pid);

        // Send SIGTERM to the process group (negative PID kills the entire group)
        let term_result = unsafe { libc::kill(-pid, libc::SIGTERM) };
        if term_result != 0 {
            log::debug!(
                "Failed to send SIGTERM to opencode process group (PID: {}): {}",
                pid,
                std::io::Error::last_os_error()
            );
        }

        // Wait for graceful shutdown (up to 2 seconds)
        let mut exited = false;
        for _ in 0..20 {
            match proc.try_wait() {
                Ok(Some(_)) => {
                    log::debug!("Opencode process terminated gracefully");
                    exited = true;
                    break;
                }
                Ok(None) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    log::warn!("Error waiting for opencode process: {}", e);
                    break;
                }
            }
        }

        // If still running, force kill with SIGKILL
        if !exited {
            log::warn!("Opencode process did not terminate gracefully, sending SIGKILL");
            let kill_result = unsafe { libc::kill(-pid, libc::SIGKILL) };
            if kill_result != 0 {
                log::warn!(
                    "Failed to send SIGKILL to opencode process group (PID: {}): {}",
                    pid,
                    std::io::Error::last_os_error()
                );
            }

            // Wait for process to exit with timeout
            for _ in 0..50 {
                match proc.try_wait() {
                    Ok(Some(_)) => {
                        log::debug!("Opencode process killed successfully");
                        exited = true;
                        break;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        log::warn!("Error waiting for opencode process after SIGKILL: {}", e);
                        break;
                    }
                }
            }
        }

        // Final wait to reap the process
        if !exited {
            match proc.wait() {
                Ok(status) => {
                    log::warn!(
                        "Opencode process exited after timeout wait (PID: {}, status: {})",
                        pid,
                        status
                    );
                    exited = true;
                }
                Err(e) => {
                    log::error!("Failed to wait for opencode process (PID: {}): {}", pid, e);
                }
            }
        }

        if !exited {
            log::error!("Failed to terminate opencode process (PID: {})", pid);
        }

        log::debug!("Opencode process cleanup completed");
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
    /// Explicitly shutdown the opencode process.
    /// This is useful for graceful shutdown scenarios.
    pub fn shutdown(&self) {
        log::info!("Shutting down opencode harness");
        self.cleanup();
    }

    pub(crate) fn new_for_test(port: u32) -> Self {
        Self {
            proc: Arc::new(Mutex::new(None)),
            opencode_client: OpencodeApiClient::new(port),
        }
    }
}

impl Drop for OpencodeHarness {
    fn drop(&mut self) {
        if Arc::strong_count(&self.proc) != 1 {
            log::debug!("OpencodeHarness dropped but shared owners remain, skipping cleanup");
            return;
        }

        log::debug!("OpencodeHarness dropped on last owner, initiating cleanup");
        self.cleanup();
    }
}
