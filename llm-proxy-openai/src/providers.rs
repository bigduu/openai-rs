use std::env;

use async_trait::async_trait;
use bytes::Bytes;
use llm_proxy_core::{
    traits::{RequestParser, TokenProvider, UrlProvider},
    ClientProvider, Error, Result,
};
use serde_json::Value;

use crate::types::ChatCompletionRequest;

/// Parser for OpenAI chat completion requests
pub struct OpenAIRequestParser;

impl OpenAIRequestParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestParser for OpenAIRequestParser {
    async fn parse(&self, body: Bytes) -> Result<(Value, bool)> {
        let request: ChatCompletionRequest = serde_json::from_slice(&body)
            .map_err(|e| Error::ParseError(format!("Failed to parse request: {e}")))?;

        let stream_requested = request.stream;
        let request_value = serde_json::to_value(request)
            .map_err(|e| Error::ParseError(format!("Failed to serialize request: {e}")))?;

        Ok((request_value, stream_requested))
    }
}
pub struct StaticClientProvider {
    client: reqwest::Client,
}

impl StaticClientProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("llm-proxy-openai")
            .build()
            .expect("Failed to create reqwest client");
        Self { client }
    }
}

#[async_trait]
impl ClientProvider for StaticClientProvider {
    async fn get_client(&self) -> Result<reqwest::Client> {
        Ok(self.client.clone())
    }
}

/// Provider that gets an OpenAI API token from an environment variable
pub struct StaticTokenProvider {
    token: String,
}

impl StaticTokenProvider {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl TokenProvider for StaticTokenProvider {
    async fn get_token(&self) -> Result<String> {
        Ok(self.token.clone())
    }
}

/// Provider that gets an OpenAI API token from an environment variable
pub struct EnvTokenProvider {
    env_var: String,
}

impl EnvTokenProvider {
    pub fn new(env_var: impl Into<String>) -> Self {
        Self {
            env_var: env_var.into(),
        }
    }

    /// Create a provider that uses the standard OPENAI_API_KEY environment variable
    pub fn standard() -> Self {
        Self::new("OPENAI_API_KEY")
    }
}

#[async_trait]
impl TokenProvider for EnvTokenProvider {
    async fn get_token(&self) -> Result<String> {
        env::var(&self.env_var).map_err(|e| {
            Error::ConfigError(format!(
                "Failed to get OpenAI API key from environment variable {}: {e}",
                self.env_var
            ))
        })
    }
}

/// Provider that returns a static URL for an OpenAI API endpoint
pub struct OpenAIUrlProvider {
    endpoint: String,
}

impl OpenAIUrlProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// Create a provider for the OpenAI chat completions endpoint
    pub fn chat_completions() -> Self {
        Self::new("https://api.openai.com/v1/chat/completions")
    }

    /// Create a provider for the OpenAI completions endpoint
    pub fn completions() -> Self {
        Self::new("https://api.openai.com/v1/completions")
    }

    /// Create a provider for the OpenAI embeddings endpoint
    pub fn embeddings() -> Self {
        Self::new("https://api.openai.com/v1/embeddings")
    }
}

#[async_trait]
impl UrlProvider for OpenAIUrlProvider {
    async fn get_url(&self) -> Result<String> {
        Ok(self.endpoint.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_parser() {
        let parser = OpenAIRequestParser::new();
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

        let bytes = serde_json::to_vec(&request).unwrap();
        let result = parser.parse(bytes.into()).await;
        assert!(result.is_ok());

        let (parsed, stream) = result.unwrap();
        assert!(stream);
        assert_eq!(parsed["model"], "gpt-4");
    }

    #[tokio::test]
    async fn test_env_token_provider() {
        let var_name = "TEST_OPENAI_KEY";
        let test_token = "test-token-123";
        env::set_var(var_name, test_token);

        let provider = EnvTokenProvider::new(var_name);
        let result = provider.get_token().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_token);

        env::remove_var(var_name);
    }

    #[tokio::test]
    async fn test_url_provider() {
        let provider = OpenAIUrlProvider::chat_completions();
        let result = provider.get_url().await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://api.openai.com/v1/chat/completions"
        );
    }
}
