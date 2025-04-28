use super::TokenProvider;
use anyhow::Result;
use async_trait::async_trait;

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
