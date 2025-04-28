//! Module responsible for forwarding processed events to the target LLM API (initially OpenAI).

use crate::event::InternalStreamEvent;
use crate::openai_types::{
    OpenAiChatCompletionRequestBuilder, OpenAiChatMessage, OpenAiChatMessageBuilder,
    OpenAiStreamChunk,
};
use crate::token_provider::TokenProvider;
use crate::url_provider::UrlProvider;
use anyhow::{Context, Result};
use bytes::Bytes;
use futures_util::{Stream, StreamExt}; // Use Stream trait from futures_util
use reqwest::{self};
use serde_json::Value;
use std::pin::Pin;

pub struct StreamForwarder {
    client: reqwest::Client,
    // Could add configuration here, e.g., target model, API base URL
    model_name: String,
}

impl StreamForwarder {
    /// Creates a new StreamForwarder.
    pub fn new(client: reqwest::Client, model_name: String) -> Self {
        StreamForwarder { client, model_name }
    }

    /// Convert internal events to OpenAI chat messages.
    ///
    /// This method filters and transforms a vector of `InternalStreamEvent`s into OpenAI's
    /// chat message format, skipping any events that lack either role or content.
    ///
    /// # Arguments
    ///
    /// * `events` - A vector of `InternalStreamEvent`s to convert
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `OpenAiChatMessage`s, or an error if no valid messages were found.
    fn convert_to_openai_messages(
        &self,
        events: Vec<InternalStreamEvent>,
    ) -> Result<Vec<OpenAiChatMessage>> {
        let messages: Vec<OpenAiChatMessage> = events
            .into_iter()
            .filter_map(|event| match (event.role, event.content) {
                (Some(role), Some(content)) => OpenAiChatMessageBuilder::default()
                    .role(role)
                    .content(content)
                    .build()
                    .ok(),
                _ => None,
            })
            .collect();

        if messages.is_empty() {
            return Err(anyhow::anyhow!("No valid messages to forward"));
        }

        Ok(messages)
    }

    /// Build the OpenAI chat completion request.
    ///
    /// Creates a request body for the OpenAI chat completions API using the provided messages
    /// and the configured model name. The stream parameter is always set to true.
    ///
    /// # Arguments
    ///
    /// * `messages` - A vector of `OpenAiChatMessage`s to include in the request
    ///
    /// # Returns
    ///
    /// A `Result` containing the OpenAI chat completion request
    fn build_chat_request(&self, messages: Vec<OpenAiChatMessage>) -> Result<Value> {
        let request = OpenAiChatCompletionRequestBuilder::default()
            .model(self.model_name.clone())
            .messages(messages)
            .stream(true)
            .build()
            .context("Failed to build OpenAI request body")?;

        serde_json::to_value(request).context("Failed to serialize request to JSON")
    }

    /// Send request to OpenAI API and validate response.
    ///
    /// Makes a POST request to the OpenAI chat completions API and validates the response.
    /// If the response status is not successful, returns an error with the status and error body.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token for the API
    /// * `request_body` - The serialized request body
    /// * `url_provider` - The URL provider to get the API endpoint
    ///
    /// # Returns
    ///
    /// A `Result` containing the API response if successful
    async fn send_request(
        &self,
        token: String,
        request_body: Value,
        url_provider: &dyn UrlProvider,
    ) -> Result<reqwest::Response> {
        let url = url_provider
            .get_url()
            .await
            .context("Failed to get API URL")?;

        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&request_body)
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

    /// Process the SSE stream from OpenAI into internal events.
    ///
    /// Converts the byte stream from OpenAI's SSE format into a stream of internal events.
    /// Handles parsing of SSE data lines, JSON deserialization, and error propagation.
    ///
    /// # Arguments
    ///
    /// * `byte_stream` - The raw byte stream from the OpenAI API
    ///
    /// # Returns
    ///
    /// A stream that yields `Result<InternalStreamEvent>`
    fn process_stream(
        byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send,
    ) -> impl Stream<Item = Result<InternalStreamEvent>> {
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
                                    break;
                                }
                                match serde_json::from_str::<OpenAiStreamChunk>(data) {
                                    Ok(chunk_data) => {
                                        if let Some(choice) = chunk_data.choices.into_iter().next() {
                                            if let Some(content) = choice.delta.content {
                                                events.push(Ok(InternalStreamEvent::new_assistant(content)));
                                            }
                                        }
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

    /// Forwards the processed events to the OpenAI API and returns a stream of response events.
    ///
    /// This method orchestrates the entire process of:
    /// 1. Converting internal events to OpenAI format
    /// 2. Building the API request
    /// 3. Sending the request and getting the response
    /// 4. Processing the streaming response
    ///
    /// # Arguments
    ///
    /// * `events` - A vector of `InternalStreamEvent`s representing the conversation history/prompt
    /// * `token_provider` - A `TokenProvider` implementation to get the authentication token
    /// * `url_provider` - A `UrlProvider` implementation to get the API endpoint
    ///
    /// # Returns
    ///
    /// A `Result` containing a pinned box stream that yields `Result<InternalStreamEvent>`
    pub async fn forward(
        &self,
        events: Vec<InternalStreamEvent>,
        token_provider: &dyn TokenProvider,
        url_provider: &dyn UrlProvider,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<InternalStreamEvent>> + Send>>> {
        let messages = self.convert_to_openai_messages(events)?;
        let request_body = self.build_chat_request(messages)?;
        let token = token_provider
            .get_token()
            .await
            .context("Failed to get authentication token")?;
        let response = self.send_request(token, request_body, url_provider).await?;
        let byte_stream = response.bytes_stream();
        let event_stream = Self::process_stream(byte_stream);

        Ok(Box::pin(event_stream))
    }
}
