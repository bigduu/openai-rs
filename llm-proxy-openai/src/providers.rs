use std::env;

use async_trait::async_trait;
use bytes::Bytes;
use llm_proxy_core::{ClientProvider, Error, RequestParser, Result, TokenProvider, UrlProvider};

use crate::ChatCompletionRequest;

/// Parser for `OpenAI` chat completion requests
pub struct OpenAIRequestParser;

impl OpenAIRequestParser {
    /// Create a new `OpenAI` request parser with the given route configuration
    pub const fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RequestParser<ChatCompletionRequest> for OpenAIRequestParser {
    async fn parse(&self, body: Bytes) -> Result<ChatCompletionRequest> {
        let request: ChatCompletionRequest = serde_json::from_slice(&body)
            .map_err(|e| Error::ParseError(format!("Failed to parse request JSON: {e}")))?;

        Ok(request)
    }
}
pub struct StaticClientProvider {
    client: reqwest::Client,
}

impl StaticClientProvider {
    /// Create a new `StaticClientProvider` with a `reqwest` client
    ///
    /// # Panics
    ///
    /// This function will panic if the `reqwest` client cannot be created.
    #[must_use]
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("llm-proxy-openai")
            .build()
            .expect("Failed to create reqwest client");
        Self { client }
    }
}

impl Default for StaticClientProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClientProvider for StaticClientProvider {
    async fn get_client(&self) -> Result<reqwest::Client> {
        Ok(self.client.clone())
    }
}

/// Provider that gets an `OpenAI` API token from an environment variable
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

/// Provider that gets an `OpenAI` API token from an environment variable
pub struct EnvTokenProvider {
    env_var: String,
}

impl EnvTokenProvider {
    pub fn new(env_var: impl Into<String>) -> Self {
        Self {
            env_var: env_var.into(),
        }
    }

    /// Create a provider that uses the standard `OPENAI_API_KEY` environment variable
    #[must_use]
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

/// Provider that returns a static URL for an `OpenAI` API endpoint
pub struct OpenAIUrlProvider {
    endpoint: String,
}

impl OpenAIUrlProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// Create a provider for the `OpenAI` chat completions endpoint
    #[must_use]
    pub fn chat_completions() -> Self {
        Self::new("https://api.openai.com/v1/chat/completions")
    }

    /// Create a provider for the `OpenAI` completions endpoint
    #[must_use]
    pub fn completions() -> Self {
        Self::new("https://api.openai.com/v1/completions")
    }

    /// Create a provider for the `OpenAI` embeddings endpoint
    #[must_use]
    pub fn embeddings() -> Self {
        Self::new("https://api.openai.com/v1/embeddings")
    }
}

impl UrlProvider for OpenAIUrlProvider {
    fn get_url(&self) -> Result<String> {
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

        let bytes = serde_json::to_vec(&request).expect("Failed to serialize request");
        let result = parser.parse(bytes.into()).await;
        assert!(result.is_ok());

        let parsed = result.expect("Failed to parse request");
        assert!(parsed.stream);
        assert_eq!(parsed.model, "gpt-4");
    }

    #[tokio::test]
    async fn test_env_token_provider() {
        let var_name = "TEST_OPENAI_KEY";
        let test_token = "test-token-123";
        env::set_var(var_name, test_token);

        let provider = EnvTokenProvider::new(var_name);
        let result = provider.get_token().await;
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to get token"), test_token);

        env::remove_var(var_name);
    }

    #[tokio::test]
    async fn test_url_provider() {
        let provider = OpenAIUrlProvider::chat_completions();
        let result = provider.get_url().await;
        assert!(result.is_ok());
        assert_eq!(
            result.expect("Failed to get URL"),
            "https://api.openai.com/v1/chat/completions"
        );
    }
}
