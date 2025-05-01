use anyhow::Result;
use bytes::Bytes;
use tokio::sync::mpsc;

use crate::forwarder::StreamMessage;

/// A trait for providers that can convert OpenAI stream events to SSE format
#[async_trait::async_trait]
pub trait SseProvider: Send + Sync {
    /// Converts a stream of OpenAI events to SSE format
    async fn to_sse_channel(
        &self,
        rx: mpsc::Receiver<StreamMessage>,
    ) -> Result<mpsc::Receiver<Result<Bytes>>>;
}

pub mod default_sse;
