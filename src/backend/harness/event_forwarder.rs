use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use futures::StreamExt;
use tokio::{
    sync::broadcast,
    time::{Duration, sleep},
};
use uuid::Uuid;

use crate::backend::{
    BackendContext,
    db::{Database, sqlite::Sqlite},
    harness::{Harness, OpencodeEventPayload, OpencodeGlobalEvent, opencode::OpencodeHarness},
    repo::message_events::{MessageDiffEvent, MessageEventApplier, MessageEventApplyError},
};

const EVENT_RETRY_DELAY_MS: u64 = 1000;

type SenderMap = Arc<Mutex<HashMap<Uuid, broadcast::Sender<MessageDiffEvent>>>>;

pub fn spawn_event_forwarder(
    ctx: BackendContext<Sqlite>,
    harness: OpencodeHarness,
    sender_by_session_id: SenderMap,
    shutdown: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("event forwarder runtime should initialize");

        runtime.block_on(async move {
            let applier = MessageEventApplier::new(ctx.clone());
            let mut pending_part_events: HashMap<String, Vec<OpencodeGlobalEvent>> = HashMap::new();

            loop {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                if let Err(err) = reconcile_all_sessions(
                    &ctx,
                    &harness,
                    &applier,
                    &sender_by_session_id,
                    &mut pending_part_events,
                )
                .await
                {
                    log::warn!("event_forwarder reconcile failed: {err}");
                }

                let stream = match harness.get_event_stream().await {
                    Ok(stream) => stream,
                    Err(err) => {
                        log::warn!("event_forwarder connect failed: {err}");
                        sleep(Duration::from_millis(EVENT_RETRY_DELAY_MS)).await;
                        continue;
                    }
                };

                futures::pin_mut!(stream);

                loop {
                    if shutdown.load(Ordering::SeqCst) {
                        break;
                    }

                    let item = tokio::select! {
                        next = stream.next() => next,
                        _ = sleep(Duration::from_millis(250)) => continue,
                    };

                    let Some(item) = item else {
                        break;
                    };

                    match item {
                        Ok(evt) => match serde_json::from_str::<OpencodeGlobalEvent>(&evt.data) {
                            Ok(event) => {
                                handle_event(
                                    &applier,
                                    event,
                                    &sender_by_session_id,
                                    &mut pending_part_events,
                                )
                                .await;
                            }
                            Err(err) => {
                                log::warn!(
                                    "event_forwarder parse failed: {err}; payload={}",
                                    evt.data
                                )
                            }
                        },
                        Err(err) => {
                            log::warn!("event_forwarder stream dropped: {err}");
                            break;
                        }
                    }
                }

                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                sleep(Duration::from_millis(EVENT_RETRY_DELAY_MS)).await;
            }
        });
    })
}

async fn reconcile_all_sessions(
    ctx: &BackendContext<Sqlite>,
    harness: &OpencodeHarness,
    applier: &MessageEventApplier<Sqlite>,
    sender_by_session_id: &SenderMap,
    pending_part_events: &mut HashMap<String, Vec<OpencodeGlobalEvent>>,
) -> Result<(), String> {
    let sessions = ctx
        .db
        .list_sessions_with_harness_ids()
        .await
        .map_err(|e| e.to_string())?;

    for session in sessions {
        let Some(harness_session_id) = session.harness_session_id.clone() else {
            continue;
        };

        let messages = harness
            .get_session_messages(&harness_session_id, None, session.dir.as_deref())
            .await
            .map_err(|e| e.to_string())?;

        for message in messages {
            let harness_message_id = message.id().to_string();
            match applier.apply_message_with_parts(session.id, message).await {
                Ok(diffs) => {
                    for diff in diffs {
                        emit_diff(sender_by_session_id, diff.clone());
                    }
                    flush_pending(
                        applier,
                        &harness_message_id,
                        sender_by_session_id,
                        pending_part_events,
                    )
                    .await;
                }
                Err(err) => log::warn!(
                    "event_forwarder reconcile apply failed for session {}: {}",
                    session.id,
                    err
                ),
            }
        }
    }

    Ok(())
}

async fn handle_event(
    applier: &MessageEventApplier<Sqlite>,
    event: OpencodeGlobalEvent,
    sender_by_session_id: &SenderMap,
    pending_part_events: &mut HashMap<String, Vec<OpencodeGlobalEvent>>,
) {
    let pending_key = part_pending_key(&event);

    match applier.apply(event.clone()).await {
        Ok(Some(diff)) => {
            if let Some(harness_message_id) = message_ready_key(&diff) {
                flush_pending(
                    applier,
                    &harness_message_id,
                    sender_by_session_id,
                    pending_part_events,
                )
                .await;
            }
            emit_diff(sender_by_session_id, diff);
        }
        Ok(None) => {}
        Err(MessageEventApplyError::MessageNotFound(_)) => {
            if let Some(message_key) = pending_key {
                pending_part_events
                    .entry(message_key)
                    .or_default()
                    .push(event);
            }
        }
        Err(err) => log::warn!("event_forwarder apply failed: {err}"),
    }
}

async fn flush_pending(
    applier: &MessageEventApplier<Sqlite>,
    harness_message_id: &str,
    sender_by_session_id: &SenderMap,
    pending_part_events: &mut HashMap<String, Vec<OpencodeGlobalEvent>>,
) {
    let key = harness_message_id.to_string();
    let Some(events) = pending_part_events.remove(&key) else {
        return;
    };

    for event in events {
        match applier.apply(event).await {
            Ok(Some(diff)) => emit_diff(sender_by_session_id, diff),
            Ok(None) => {}
            Err(err) => log::warn!("event_forwarder pending apply failed: {err}"),
        }
    }
}

fn emit_diff(sender_by_session_id: &SenderMap, diff: MessageDiffEvent) {
    let session_id = match &diff {
        MessageDiffEvent::MessageUpserted { session_id, .. }
        | MessageDiffEvent::MessagePartUpserted { session_id, .. }
        | MessageDiffEvent::MessageRemoved { session_id, .. }
        | MessageDiffEvent::SessionIdle { session_id } => *session_id,
    };

    let maybe_sender = sender_by_session_id
        .lock()
        .ok()
        .and_then(|map| map.get(&session_id).cloned());
    if let Some(sender) = maybe_sender {
        let _ = sender.send(diff);
    }
}

fn part_pending_key(event: &OpencodeGlobalEvent) -> Option<String> {
    match &event.payload {
        OpencodeEventPayload::MessagePartUpdated { props } => {
            Some(props.part.message_id().to_string())
        }
        _ => None,
    }
}

fn message_ready_key(diff: &MessageDiffEvent) -> Option<String> {
    match diff {
        MessageDiffEvent::MessageUpserted { message, .. } => message.harness_message_id.clone(),
        _ => None,
    }
}
