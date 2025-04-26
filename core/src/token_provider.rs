use anyhow::Result;
use async_trait::async_trait;

/// Defines the contract for providing authentication tokens.
///
/// Implementations of this trait are responsible for retrieving the necessary
/// credentials (like API keys, OAuth tokens, etc.) required to authenticate
/// requests with downstream LLM APIs (e.g., OpenAI, Claude).
///
/// This abstraction allows the core forwarding logic to remain decoupled from
/// specific authentication mechanisms. Different strategies (static keys,
/// dynamic token refresh, caching) can be implemented and swapped easily.
#[async_trait]
pub trait TokenProvider: Send + Sync {
    /// Asynchronously retrieves an authentication token.
    ///
    /// # Returns
    ///
    /// * `Ok(String)`: Containing the authentication token if retrieval is successful.
    /// * `Err(anyhow::Error)`: If an error occurred during token retrieval (e.g.,
    ///   failed network request for dynamic tokens, missing configuration).
    async fn get_token(&self) -> Result<String>;
}

/// A simple `TokenProvider` that always returns a fixed, pre-configured token.
/// Useful for scenarios where the API key or token does not change, like
/// standard OpenAI API keys provided via configuration.
#[derive(Clone)] // Clone is useful if the provider needs to be shared or passed around
pub struct StaticTokenProvider {
    token: String,
}

impl StaticTokenProvider {
    /// Creates a new `StaticTokenProvider`.
    ///
    /// # Arguments
    ///
    /// * `token`: The static token string to be returned by `get_token`.
    pub fn new(token: String) -> Self {
        StaticTokenProvider { token }
    }
}

#[async_trait]
impl TokenProvider for StaticTokenProvider {
    /// Returns the pre-configured static token.
    /// This implementation does not involve any I/O or complex logic,
    /// so it returns immediately with the stored token.
    async fn get_token(&self) -> Result<String> {
        Ok(self.token.clone())
    }
}

// TODO: Implement DynamicTokenProvider, CacheTokenProvider, ChainedTokenProvider later.
