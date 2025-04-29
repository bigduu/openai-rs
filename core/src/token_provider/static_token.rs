use super::TokenProvider;
use anyhow::Result;
use async_trait::async_trait;

/// A simple `TokenProvider` that always returns a fixed, pre-configured token.
/// Useful for scenarios where the API key or token does not change, like
/// standard OpenAI API keys provided via configuration.
///
/// # Example
/// ```rust
/// use core::token_provider::{TokenProvider, StaticTokenProvider};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create a new static token provider
///     let provider = StaticTokenProvider::new("my-static-token".to_string());
///
///     // Get the token
///     let token = provider.get_token().await?;
///     assert_eq!(token, "my-static-token");
///     Ok(())
/// }
/// ```
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
    ///
    /// # Example
    /// ```rust
    /// use core::token_provider::StaticTokenProvider;
    ///
    /// let provider = StaticTokenProvider::new("my-api-key".to_string());
    /// ```
    pub fn new(token: String) -> Self {
        StaticTokenProvider { token }
    }
}

#[async_trait]
impl TokenProvider for StaticTokenProvider {
    /// Returns the pre-configured static token.
    /// This implementation does not involve any I/O or complex logic,
    /// so it returns immediately with the stored token.
    ///
    /// # Example
    /// ```rust
    /// use core::token_provider::{TokenProvider, StaticTokenProvider};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let provider = StaticTokenProvider::new("test-token".to_string());
    ///     let token = provider.get_token().await?;
    ///     assert_eq!(token, "test-token");
    ///     Ok(())
    /// }
    /// ```
    async fn get_token(&self) -> Result<String> {
        Ok(self.token.clone())
    }
}
