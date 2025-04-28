mod static_token;
pub use static_token::StaticTokenProvider;

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
