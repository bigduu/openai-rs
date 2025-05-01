use super::SseProvider;
use crate::forwarder::StreamMessage;
use anyhow::Result;
use bytes::Bytes;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// A default implementation of SseProvider that converts OpenAI stream events to SSE format
#[derive(Clone)]
pub struct DefaultSseProvider;

impl DefaultSseProvider {
    pub fn new() -> Self {
        DefaultSseProvider
    }
}

#[async_trait::async_trait]
impl SseProvider for DefaultSseProvider {
    async fn to_sse_channel(
        &self,
        mut rx: mpsc::Receiver<StreamMessage>,
    ) -> Result<mpsc::Receiver<Result<Bytes>>> {
        info!("Starting SSE conversion");
        let (tx, output_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                debug!("Converting message to SSE format");
                let result = match message {
                    StreamMessage::Chunk(event) => match event {
                        crate::openai_types::StreamEvent::Chunk(chunk) => {
                            let json = match serde_json::to_string(&chunk) {
                                Ok(j) => j,
                                Err(e) => {
                                    error!(error = %e, "Failed to serialize event to JSON");
                                    continue;
                                }
                            };
                            debug!(json = %json, "Created SSE chunk");
                            Ok(Bytes::from(format!("data: {}\n\n", json)))
                        }
                        crate::openai_types::StreamEvent::Done => {
                            debug!("Converting DONE message");
                            Ok(Bytes::from("data: [DONE]\n\n"))
                        }
                    },
                    StreamMessage::Done => {
                        debug!("Converting DONE message");
                        Ok(Bytes::from("data: [DONE]\n\n"))
                    }
                    StreamMessage::Error(e) => {
                        error!(error = %e, "Converting error message");
                        let err_json = json!({"error": e.to_string()});
                        Ok(Bytes::from(format!("event: error\ndata: {}\n\n", err_json)))
                    }
                };

                if tx.send(result).await.is_err() {
                    warn!("Failed to send SSE message - receiver dropped");
                    break;
                }
            }
            info!("SSE conversion completed");
        });

        Ok(output_rx)
    }
}
