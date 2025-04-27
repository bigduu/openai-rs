use anyhow::Result;
use bytes::Bytes;
use domain::event::InternalStreamEvent;
use futures_util::Stream;
use reqwest::Client;
use std::collections::VecDeque;

/// Handles streaming requests to external services
pub struct StreamDispatcher {
    client: Client,
}

impl StreamDispatcher {
    /// Creates a new StreamDispatcher
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Creates a new StreamDispatcher with a custom client
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    /// Dispatches a series of events to an external service and returns a stream of responses
    pub async fn dispatch(
        &self,
        events: VecDeque<InternalStreamEvent>,
        token: String,
        endpoint: String,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>> {
        let response = self
            .client
            .post(&endpoint)
            .bearer_auth(token)
            .json(&events)
            .send()
            .await?;

        Ok(response.bytes_stream())
    }

    /// Default implementation that can be changed when needed
    pub fn default() -> Self {
        Self::new()
    }
}

impl Default for StreamDispatcher {
    fn default() -> Self {
        Self::default()
    }
}
