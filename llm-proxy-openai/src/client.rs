use std::clone::Clone;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use llm_proxy_core::{
    ClientProvider, Error, LLMClient, RequestParser, Result, TokenProvider, UrlProvider,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::types::{ChatCompletionRequest, ErrorResponse, StreamChunk};

/// Parser for `OpenAI` chat completion requests
pub struct OpenAIRequestParser;

#[async_trait]
impl RequestParser<ChatCompletionRequest> for OpenAIRequestParser {
    async fn parse(&self, body: Bytes) -> Result<ChatCompletionRequest> {
        let request: ChatCompletionRequest = serde_json::from_slice(&body)
            .map_err(|e| Error::ParseError(format!("Failed to parse OpenAI request: {e}")))?;
        Ok(request)
    }
}

/// OpenAI-specific implementation of `LLMClient`
pub struct OpenAIClient {
    client_provider: Arc<dyn ClientProvider>,
    token_provider: Arc<dyn TokenProvider>,
    url_provider: Arc<dyn UrlProvider>,
    request_parser: OpenAIRequestParser,
}

impl Clone for OpenAIClient {
    fn clone(&self) -> Self {
        Self {
            client_provider: self.client_provider.clone(),
            token_provider: self.token_provider.clone(),
            url_provider: self.url_provider.clone(),
            request_parser: OpenAIRequestParser,
        }
    }
}

impl OpenAIClient {
    /// Create a new `OpenAI` client with the given providers
    pub fn new(
        client_provider: Arc<dyn ClientProvider>,
        token_provider: Arc<dyn TokenProvider>,
        url_provider: Arc<dyn UrlProvider>,
    ) -> Self {
        Self {
            client_provider,
            token_provider,
            url_provider,
            request_parser: OpenAIRequestParser,
        }
    }

    /// Send request to `OpenAI` and get response
    async fn send_request(
        &self,
        request: &ChatCompletionRequest,
        client: reqwest::Client,
        token: String,
        url: String,
    ) -> Result<reqwest::Response> {
        let response = client
            .post(url)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::LLMError(format!("Failed to send request to OpenAI: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.json::<ErrorResponse>().await.map_err(|e| {
                Error::LLMError(format!(
                    "Failed to parse OpenAI error response: {e}, status: {status}"
                ))
            })?;
            return Err(Error::LLMError(format!(
                "OpenAI request failed: {} ({})",
                error_body.error.message, status
            )));
        }

        Ok(response)
    }

    /// Process a streaming response from `OpenAI`
    async fn handle_stream(
        self,
        response: reqwest::Response,
        tx: mpsc::Sender<Result<Bytes>>,
    ) -> Result<()> {
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => self.process_chunk(chunk, &tx).await?,
                Err(e) => {
                    self.send_error(&tx, format!("Error reading chunk from OpenAI: {e}"))
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Process a single chunk of data from the stream
    async fn process_chunk(&self, chunk: Bytes, tx: &mpsc::Sender<Result<Bytes>>) -> Result<()> {
        let lines = String::from_utf8_lossy(&chunk);
        debug!(chunk = %lines, "Received raw chunk");

        for line in lines.lines() {
            self.process_line(line, &chunk, tx).await?;
        }

        Ok(())
    }

    /// Process a single line from the chunk
    async fn process_line(
        &self,
        line: &str,
        original_chunk: &Bytes,
        tx: &mpsc::Sender<Result<Bytes>>,
    ) -> Result<()> {
        if !line.starts_with("data: ") {
            return Ok(());
        }

        let data = line[5..].trim();
        debug!(data = %data, "Processing data line");

        if data == "[DONE]" {
            info!("Received [DONE] signal");
            return Ok(());
        }

        self.parse_and_send_chunk(data, original_chunk, tx).await
    }

    /// Parse the chunk data and send it through the channel
    async fn parse_and_send_chunk(
        &self,
        data: &str,
        original_chunk: &Bytes,
        tx: &mpsc::Sender<Result<Bytes>>,
    ) -> Result<()> {
        match serde_json::from_str::<StreamChunk>(data) {
            Ok(chunk_data) => {
                debug!(?chunk_data, "Successfully parsed chunk");
                self.send_chunk(original_chunk, tx).await
            }
            Err(e) => {
                error!(
                    error = %e,
                    data = %data,
                    "Failed to parse OpenAI stream chunk"
                );
                self.send_error(tx, format!("Failed to parse OpenAI stream chunk: {e}"))
                    .await
            }
        }
    }

    /// Send a chunk through the channel
    async fn send_chunk(&self, chunk: &Bytes, tx: &mpsc::Sender<Result<Bytes>>) -> Result<()> {
        if tx.send(Ok(chunk.clone())).await.is_err() {
            warn!("Failed to send chunk - receiver dropped");
        }
        Ok(())
    }

    /// Send an error message through the channel
    async fn send_error(
        &self,
        tx: &mpsc::Sender<Result<Bytes>>,
        error_message: String,
    ) -> Result<()> {
        if tx.send(Err(Error::LLMError(error_message))).await.is_err() {
            warn!("Failed to send error - receiver dropped");
        }
        Ok(())
    }

    /// Process a non-streaming response from `OpenAI`
    async fn handle_non_stream(
        self,
        response: reqwest::Response,
        tx: mpsc::Sender<Result<Bytes>>,
    ) -> Result<()> {
        let bytes = response.bytes().await.map_err(|e| {
            Error::LLMError(format!("Failed to read OpenAI non-streaming response: {e}"))
        })?;

        if tx.send(Ok(bytes)).await.is_err() {
            warn!("Failed to send response - receiver dropped");
        }

        Ok(())
    }
}

#[async_trait]
impl LLMClient<ChatCompletionRequest> for OpenAIClient {
    async fn execute(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<mpsc::Receiver<Result<Bytes>>> {
        // 1. Get dependencies
        let client = self
            .client_provider
            .get_client()
            .await
            .map_err(|e| Error::LLMError(format!("Failed to get HTTP client: {e}")))?;
        let token = self
            .token_provider
            .get_token()
            .await
            .map_err(|e| Error::LLMError(format!("Failed to get API token: {e}")))?;
        let url = self
            .url_provider
            .get_url()
            .await
            .map_err(|e| Error::LLMError(format!("Failed to get API URL: {e}")))?;

        // 2. Create response channel
        let (tx, rx) = mpsc::channel(100);

        // 3. Send request and handle response
        let response = self.send_request(&request, client, token, url).await?;

        // 4. Handle response based on streaming flag
        let client = self.clone();
        let stream = request.stream;
        info!("The request is streaming: {}", stream);
        tokio::spawn(async move {
            let result = if stream {
                client.handle_stream(response, tx).await
            } else {
                client.handle_non_stream(response, tx).await
            };

            if let Err(e) = result {
                error!(error = %e, "Error handling OpenAI response");
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_proxy_core::{ClientProvider, TokenProvider, UrlProvider};

    struct MockClientProvider;
    struct MockTokenProvider;
    struct MockUrlProvider;

    #[async_trait]
    impl ClientProvider for MockClientProvider {
        async fn get_client(&self) -> Result<reqwest::Client> {
            Ok(reqwest::Client::new())
        }
    }

    #[async_trait]
    impl TokenProvider for MockTokenProvider {
        async fn get_token(&self) -> Result<String> {
            Ok("test-token".to_string())
        }
    }

    #[async_trait]
    impl UrlProvider for MockUrlProvider {
        async fn get_url(&self) -> Result<String> {
            Ok("https://api.openai.com/v1/chat/completions".to_string())
        }
    }

    #[tokio::test]
    async fn test_parse_request() {
        let client = OpenAIClient::new(
            Arc::new(MockClientProvider),
            Arc::new(MockTokenProvider),
            Arc::new(MockUrlProvider),
        );

        let request = serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "stream": true
        });

        let result = client
            .request_parser
            .parse(Bytes::from(
                serde_json::to_vec(&request).expect("Failed to serialize request"),
            ))
            .await
            .expect("Failed to parse request");

        assert_eq!(result.model, "gpt-4");
        assert_eq!(result.messages.len(), 1);
        assert!(result.stream);
    }
}
