use async_trait::async_trait;

use crate::{
    types::{ResponseStream, Result},
    LLMRequest,
};

/// Trait for interacting with an LLM service.
///
/// This trait defines the core interface for sending requests to
/// an LLM service and receiving responses. Implementations handle
/// the specifics of communicating with different LLM providers.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::{LLMClient, ResponseStream};
/// #
/// # struct OpenAIRequest;
///
/// struct OpenAIClient {
///     api_key: String,
///     base_url: String,
/// }
///
/// #[async_trait]
/// impl LLMClient<OpenAIRequest> for OpenAIClient {
///     async fn execute(&self, request: OpenAIRequest) -> Result<ResponseStream> {
///         // Send request to OpenAI API and return response stream
///         # todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait LLMClient<T: LLMRequest>: Send + Sync {
    /// Execute a request against the LLM service.
    ///
    /// # Arguments
    /// * `request` - The processed request to send to the LLM
    ///
    /// # Returns
    /// A channel receiver that will receive the response chunks
    async fn execute(&self, request: T) -> Result<ResponseStream>;
}

/// Trait for managing LLM API tokens.
///
/// This trait provides a way to abstract token management,
/// allowing different strategies for token storage and retrieval.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::TokenProvider;
///
/// struct EnvTokenProvider {
///     env_var: String,
/// }
///
/// #[async_trait]
/// impl TokenProvider for EnvTokenProvider {
///     async fn get_token(&self) -> Result<String> {
///         std::env::var(&self.env_var).map_err(Into::into)
///     }
/// }
/// ```
#[async_trait]
pub trait TokenProvider: Send + Sync {
    /// Get an API token for the LLM service.
    /// This might involve reading from environment variables,
    /// secure storage, or a token management service.
    async fn get_token(&self) -> Result<String>;
}

/// Trait for providing the LLM service URL.
///
/// This trait allows the service URL to be configured or
/// determined dynamically at runtime.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::UrlProvider;
///
/// struct ConfigUrlProvider {
///     base_url: String,
/// }
///
/// #[async_trait]
/// impl UrlProvider for ConfigUrlProvider {
///     async fn get_url(&self) -> Result<String> {
///         Ok(self.base_url.clone())
///     }
/// }
/// ```
#[async_trait]
pub trait UrlProvider: Send + Sync {
    /// Get the URL for the LLM service endpoint.
    async fn get_url(&self) -> Result<String>;
}

/// Trait for providing an HTTP client.
///
/// This trait allows the HTTP client to be configured with
/// custom settings or replaced with different implementations.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use std::time::Duration;
/// # use llm_proxy_core::ClientProvider;
///
/// struct CustomClientProvider {
///     timeout: Duration,
/// }
///
/// #[async_trait]
/// impl ClientProvider for CustomClientProvider {
///     async fn get_client(&self) -> Result<reqwest::Client> {
///         reqwest::Client::builder()
///             .timeout(self.timeout)
///             .build()
///             .map_err(Into::into)
///     }
/// }
/// ```
#[async_trait]
pub trait ClientProvider: Send + Sync {
    /// Get an HTTP client for making requests.
    async fn get_client(&self) -> Result<reqwest::Client>;
}
