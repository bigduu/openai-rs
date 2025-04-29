use super::SseProvider;
use crate::openai_types::StreamEvent;
use anyhow::Result;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde_json::json;
use std::pin::Pin;

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
    async fn to_http_stream(
        &self,
        stream: Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        let sse_stream = stream.map(|event_result| match event_result {
            Ok(event) => match event {
                StreamEvent::Chunk(chunk) => {
                    let json = serde_json::to_string(&chunk)?;
                    Ok(Bytes::from(format!("data: {}\n\n", json)))
                }
                StreamEvent::Done => Ok(Bytes::from("data: [DONE]\n\n")),
            },
            Err(e) => {
                let err_json = json!({"error": e.to_string()});
                Ok(Bytes::from(format!("event: error\ndata: {}\n\n", err_json)))
            }
        });

        Ok(Box::pin(sse_stream))
    }
}
