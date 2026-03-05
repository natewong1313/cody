use futures::StreamExt;
use std::os::unix::process::CommandExt;
use std::pin::Pin;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::backend::harness::{
    Harness, HarnessAssistantEvent, HarnessAssistantEventStream, HarnessError, HarnessMessage,
    HarnessSessionStatus, OpencodePartInput, OpencodeSendMessageRequest,
};
use crate::backend::repo::{user_message::UserMessage, user_message_part::UserMessagePart};
use crate::backend::{
    harness::opencode_client::{
        OpencodeApiClient, OpencodeCreateSessionRequest, OpencodeEventPayload, OpencodeMessage,
        OpencodeSessionStatus,
    },
    repo::session::Session,
};

#[derive(Error, Debug)]
pub enum OpencodeHarnessError {
    #[error("Failed to spawn opencode process: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("Mutex poisoned")]
    #[allow(dead_code)]
    MutexPoisoned,

    #[error("API request failed: {0}")]
    ApiRequest(#[from] anyhow::Error),

    #[error("API transport failed: {0}")]
    ApiTransport(#[from] reqwest::Error),
}

// Needs to be clonable since we pass this around in the repos
#[derive(Clone)]
pub struct OpencodeHarness {
    proc: Arc<Mutex<Option<Child>>>,
    opencode_client: OpencodeApiClient,
}

impl Harness for OpencodeHarness {
    fn new() -> anyhow::Result<Self> {
        Self::new_with_port(6767)
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
    ) -> anyhow::Result<String> {
        let request = OpencodeCreateSessionRequest {
            parent_id: None,
            title: Some(session.name),
            permission: None,
        };

        let created = self
            .opencode_client
            .create_session(Some(&request), directory)
            .await
            .map_err(OpencodeHarnessError::ApiRequest)?;

        Ok(created.id)
    }

    async fn send_message_async(
        &self,
        harness_session_id: String,
        message: UserMessage,
        message_parts: Vec<UserMessagePart>,
        directory: Option<String>,
    ) -> Result<(), HarnessError> {
        let mut request = OpencodeSendMessageRequest::from(&message);
        let mut message_parts = message_parts;
        message_parts.sort_by_key(|part| part.position);
        request.parts = message_parts
            .into_iter()
            .map(OpencodePartInput::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| HarnessError::InvalidRequest(e.to_string()))?;

        self.opencode_client
            .send_message_async(&harness_session_id, &request, directory.as_deref())
            .await
            .map_err(HarnessError::ApiRequest)?;

        Ok(())
    }

    async fn get_session_messages(
        &self,
        session_id: &str,
        limit: Option<i32>,
        directory: Option<&str>,
    ) -> Result<Vec<HarnessMessage>, HarnessError> {
        let messages = self
            .opencode_client
            .get_session_messages(session_id, limit, directory)
            .await
            .map_err(HarnessError::ApiRequest)?;

        Ok(messages
            .into_iter()
            .map(|message| HarnessMessage {
                id: message.id().to_string(),
                session_id: message.session_id().to_string(),
            })
            .collect())
    }

    async fn listen_assistant_events(
        &self,
        harness_session_id: String,
        directory: Option<String>,
    ) -> Result<HarnessAssistantEventStream, HarnessError> {
        let stream = self
            .opencode_client
            .get_event_stream(directory.as_deref())
            .await
            .map_err(HarnessError::ApiTransport)?;

        let mapped = stream.filter_map(move |item| {
            let harness_session_id = harness_session_id.clone();
            async move {
                let event = match item {
                    Ok(event) => event,
                    Err(err) => return Some(Err(stream_error("event stream error", err))),
                };

                let payload = match parse_event_payload(&event.data) {
                    Ok(Some(payload)) => payload,
                    Ok(None) => return None,
                    Err(err) => return Some(Err(err)),
                };

                map_payload_to_harness_event(payload, &harness_session_id)
            }
        });

        Ok(Box::pin(mapped))
    }

    async fn get_event_stream(
        &self,
    ) -> anyhow::Result<
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
    > {
        let stream = self
            .opencode_client
            .get_event_stream(None)
            .await
            .map_err(OpencodeHarnessError::ApiTransport)?;
        Ok(Box::pin(stream))
    }
}

fn stream_error(context: &str, err: impl std::fmt::Display) -> HarnessError {
    HarnessError::ApiRequest(anyhow::anyhow!("{context}: {err}"))
}

fn is_supported_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        "session.status"
            | "message.updated"
            | "message.part.updated"
            | "message.part.delta"
            | "session.error"
    )
}

fn parse_event_payload(data: &str) -> Result<Option<OpencodeEventPayload>, HarnessError> {
    let payload: serde_json::Value = serde_json::from_str(data).map_err(|err| {
        HarnessError::ApiRequest(anyhow::anyhow!(
            "failed to parse opencode event JSON: {err}; data={data}"
        ))
    })?;

    let event_type = payload
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if !is_supported_event_type(event_type) {
        return Ok(None);
    }

    serde_json::from_value(payload).map(Some).map_err(|err| {
        HarnessError::ApiRequest(anyhow::anyhow!("failed to decode opencode event: {err}"))
    })
}

fn map_payload_to_harness_event(
    payload: OpencodeEventPayload,
    harness_session_id: &str,
) -> Option<Result<HarnessAssistantEvent, HarnessError>> {
    match payload {
        OpencodeEventPayload::SessionStatus { props } => {
            if props.session_id != harness_session_id {
                return None;
            }
            Some(Ok(HarnessAssistantEvent::SessionStatus {
                harness_session_id: props.session_id,
                status: map_session_status(props.status),
            }))
        }
        OpencodeEventPayload::MessageUpdated { props } => {
            map_message_updated(props.info, harness_session_id).map(Ok)
        }
        OpencodeEventPayload::MessagePartUpdated { props } => {
            map_message_part_updated(props.part, harness_session_id).map(Ok)
        }
        OpencodeEventPayload::MessagePartDelta { props } => {
            if props.session_id != harness_session_id {
                return None;
            }
            Some(Ok(HarnessAssistantEvent::MessagePartDelta {
                harness_session_id: props.session_id,
                message_id: props.message_id,
                part_id: props.part_id,
                field: props.field,
                delta: props.delta,
            }))
        }
        OpencodeEventPayload::SessionError { props } => {
            if let Some(session_id) = &props.session_id
                && session_id != harness_session_id
            {
                return None;
            }
            Some(Ok(HarnessAssistantEvent::SessionError {
                harness_session_id: props.session_id,
                error: props.error.to_string(),
            }))
        }
        _ => None,
    }
}

fn map_session_status(status: OpencodeSessionStatus) -> HarnessSessionStatus {
    match status {
        OpencodeSessionStatus::Idle => HarnessSessionStatus::Idle,
        OpencodeSessionStatus::Busy => HarnessSessionStatus::Busy,
        OpencodeSessionStatus::Retry {
            attempt,
            message,
            next,
        } => HarnessSessionStatus::Retry {
            attempt,
            message,
            next,
        },
    }
}

fn map_message_updated(
    message: OpencodeMessage,
    harness_session_id: &str,
) -> Option<HarnessAssistantEvent> {
    match message {
        OpencodeMessage::Assistant(assistant) if assistant.session_id == harness_session_id => {
            Some(HarnessAssistantEvent::MessageUpdated {
                harness_session_id: assistant.session_id,
                message_id: assistant.id,
                completed_at: assistant.time.completed,
                error: assistant
                    .error
                    .and_then(|err| serde_json::to_string(&err).ok()),
            })
        }
        _ => None,
    }
}

fn map_message_part_updated(
    part: crate::backend::harness::opencode_client::OpencodePart,
    harness_session_id: &str,
) -> Option<HarnessAssistantEvent> {
    if part.session_id() != harness_session_id {
        return None;
    }

    Some(HarnessAssistantEvent::MessagePartUpdated {
        harness_session_id: part.session_id().to_string(),
        message_id: part.message_id().to_string(),
        part_id: part.id().to_string(),
        part_type: part.part_type().to_string(),
        payload: serde_json::to_value(part).unwrap_or(serde_json::Value::Null),
    })
}

impl OpencodeHarness {
    fn new_with_port(port: u32) -> anyhow::Result<Self> {
        log::debug!("Starting opencode on port {port}");
        let proc = unsafe {
            Command::new("opencode")
                .arg("serve")
                .arg("--port")
                .arg(port.to_string())
                .arg("--print-logs")
                .arg("--log-level")
                .arg("DEBUG")
                .pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                })
                .spawn()
                .map_err(OpencodeHarnessError::Spawn)?
        };

        log::debug!("Opencode running on port {port}");

        let opencode_client = OpencodeApiClient::new(port);

        Ok(Self {
            proc: Arc::new(Mutex::new(Some(proc))),
            opencode_client,
        })
    }
}

#[cfg(test)]
impl OpencodeHarness {
    /// Explicitly shutdown the opencode process.
    /// This is useful for graceful shutdown scenarios.
    #[allow(dead_code)]
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

    pub(crate) fn new_with_process_for_test(port: u32) -> anyhow::Result<Self> {
        Self::new_with_port(port)
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
