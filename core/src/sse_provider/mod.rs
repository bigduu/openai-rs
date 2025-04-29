use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;
use std::pin::Pin;

use crate::openai_types::StreamEvent;

/// A trait for providers that can convert OpenAI stream events to SSE format
#[async_trait]
pub trait SseProvider: Send + Sync {
    /// Converts a stream of OpenAI events to SSE format and then to HTTP stream
    async fn to_http_stream(
        &self,
        stream: Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;
}

pub mod default_sse;
