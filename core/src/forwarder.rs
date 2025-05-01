//! Module responsible for forwarding processed events to the target LLM API (initially OpenAI).

use crate::{
    client_provider::ClientProvider,
    openai_types::{OpenAiChatCompletionRequest, OpenAiStreamChunk, StreamEvent},
    token_provider::TokenProvider,
    url_provider::UrlProvider,
};
use anyhow::{Context, Result};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub enum StreamMessage {
    Chunk(StreamEvent),
    Done,
    Error(anyhow::Error),
}

pub struct StreamForwarder {
    client_provider: Arc<dyn ClientProvider>,
}

impl StreamForwarder {
    pub fn new(client_provider: Arc<dyn ClientProvider>) -> Self {
        StreamForwarder { client_provider }
    }

    pub async fn forward(
        &self,
        request: OpenAiChatCompletionRequest,
        token_provider: &dyn TokenProvider,
        url_provider: &dyn UrlProvider,
        tx: mpsc::Sender<StreamMessage>,
    ) -> Result<()> {
        debug!(?request, "Starting forward request");

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

        info!(status = %response.status(), "Got response");

        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let lines = String::from_utf8_lossy(&chunk);
                    debug!(chunk = %lines, "Received raw chunk");
                    for line in lines.lines() {
                        if line.starts_with("data:") {
                            let data = line[5..].trim();
                            debug!(data = %data, "Processing data line");
                            if data == "[DONE]" {
                                info!("Received [DONE] signal");
                                if tx.send(StreamMessage::Done).await.is_err() {
                                    warn!("Failed to send DONE message - receiver dropped");
                                    return Ok(());
                                }
                                break;
                            }
                            match serde_json::from_str::<OpenAiStreamChunk>(data) {
                                Ok(chunk_data) => {
                                    debug!(?chunk_data, "Successfully parsed chunk");
                                    if tx
                                        .send(StreamMessage::Chunk(StreamEvent::Chunk(chunk_data)))
                                        .await
                                        .is_err()
                                    {
                                        warn!("Failed to send chunk - receiver dropped");
                                        return Ok(());
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        error = %e,
                                        data = %data,
                                        "Failed to parse OpenAI stream chunk JSON"
                                    );
                                    if tx
                                        .send(StreamMessage::Error(anyhow::anyhow!(
                                            "Failed to parse OpenAI stream chunk: {}",
                                            e
                                        )))
                                        .await
                                        .is_err()
                                    {
                                        warn!("Failed to send error message - receiver dropped");
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Error reading chunk");
                    if tx
                        .send(StreamMessage::Error(anyhow::anyhow!(
                            "Error reading chunk from OpenAI stream: {}",
                            e
                        )))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send error message - receiver dropped");
                        return Ok(());
                    }
                }
            }
        }

        info!("Forward completed successfully");
        Ok(())
    }

    async fn send_request(
        &self,
        client: reqwest::Client,
        token: String,
        request: OpenAiChatCompletionRequest,
        url_provider: &dyn UrlProvider,
    ) -> Result<reqwest::Response> {
        let url = url_provider
            .get_url()
            .await
            .context("Failed to get API URL")?;

        info!(url = %url, "Sending request");

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
            error!(
                status = %status,
                body = %error_body,
                "Request failed"
            );
            return Err(anyhow::anyhow!(
                "OpenAI API request failed with status {}: {}",
                status,
                error_body
            ));
        }

        Ok(response)
    }
}
