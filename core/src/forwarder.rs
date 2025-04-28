//! Module responsible for forwarding processed events to the target LLM API (initially OpenAI).

use crate::client_provider::ClientProvider;
use crate::openai_types::{OpenAiChatCompletionRequest, OpenAiStreamChunk, StreamEvent};
use crate::token_provider::TokenProvider;
use crate::url_provider::UrlProvider;
use anyhow::{Context, Result};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use reqwest::{self, Client};
use std::pin::Pin;

pub struct StreamForwarder {
    client_provider: Box<dyn ClientProvider>,
}

impl StreamForwarder {
    pub fn new(client_provider: Box<dyn ClientProvider>) -> Self {
        StreamForwarder { client_provider }
    }

    pub async fn forward(
        &self,
        request: OpenAiChatCompletionRequest,
        token_provider: &dyn TokenProvider,
        url_provider: &dyn UrlProvider,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let token = token_provider
            .get_token()
            .await
            .context("Failed to get authentication token")?;
        let client = self
            .client_provider
            .get_client()
            .await
            .context("Failed to get HTTP client")?;
        let response = self
            .send_request(client, token, request, url_provider)
            .await?;
        let byte_stream = response.bytes_stream();
        let event_stream = Self::process_stream(byte_stream);

        Ok(Box::pin(event_stream))
    }

    async fn send_request(
        &self,
        client: Client,
        token: String,
        request: OpenAiChatCompletionRequest,
        url_provider: &dyn UrlProvider,
    ) -> Result<reqwest::Response> {
        let url = url_provider
            .get_url()
            .await
            .context("Failed to get API URL")?;

        let response = client
            .post(url)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow::anyhow!(
                "OpenAI API request failed with status {}: {}",
                status,
                error_body
            ));
        }

        Ok(response)
    }

    fn process_stream(
        byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send,
    ) -> impl Stream<Item = Result<StreamEvent>> {
        byte_stream
            .map(|chunk_result| {
                chunk_result
                    .map_err(|e| anyhow::anyhow!("Error reading chunk from OpenAI stream: {}", e))
            })
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        let lines = String::from_utf8_lossy(&chunk);
                        let mut events = Vec::new();
                        for line in lines.lines() {
                            if line.starts_with("data:") {
                                let data = line[5..].trim();
                                if data == "[DONE]" {
                                    events.push(Ok(StreamEvent::Done));
                                    break;
                                }
                                match serde_json::from_str::<OpenAiStreamChunk>(data) {
                                    Ok(chunk_data) => {
                                        events.push(Ok(StreamEvent::Chunk(chunk_data)));
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to parse OpenAI stream chunk JSON: {:?}, data: '{}'",
                                            e, data
                                        );
                                        events.push(Err(anyhow::anyhow!(
                                            "Failed to parse OpenAI stream chunk: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                        }
                        futures_util::stream::iter(events)
                    }
                    Err(e) => futures_util::stream::iter(vec![Err(e)]),
                }
            })
            .flatten()
    }
}
