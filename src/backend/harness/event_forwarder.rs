use futures::StreamExt;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

use crate::backend::{
    BackendContext,
    db::sqlite::Sqlite,
    harness::{Harness, OpencodeGlobalEvent, opencode::OpencodeHarness},
    repo::message_events::{MessageDiffEvent, MessageEventApplier},
};

const EVENT_RETRY_DELAY_MS: u64 = 1000;

pub fn spawn_event_forwarder(
    ctx: BackendContext<Sqlite>,
    harness: OpencodeHarness,
    sender: broadcast::Sender<MessageDiffEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("event forwarder runtime should initialize");

        runtime.block_on(async move {
            let applier = MessageEventApplier::new(ctx.clone());

            loop {
                let stream = match harness.get_event_stream().await {
                    Ok(stream) => stream,
                    Err(err) => {
                        log::warn!("event_forwarder connect failed: {err}");
                        sleep(Duration::from_millis(EVENT_RETRY_DELAY_MS)).await;
                        continue;
                    }
                };

                futures::pin_mut!(stream);

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(evt) => match serde_json::from_str::<OpencodeGlobalEvent>(&evt.data) {
                            Ok(event) => match applier.apply(event).await {
                                Ok(Some(diff)) => {
                                    let _ = sender.send(diff);
                                }
                                Ok(None) => {}
                                Err(err) => {
                                    log::warn!("event_forwarder apply failed: {err}");
                                }
                            },
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

                sleep(Duration::from_millis(EVENT_RETRY_DELAY_MS)).await;
            }
        });
    })
}
