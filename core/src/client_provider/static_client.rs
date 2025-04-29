use super::ClientProvider;
use anyhow::Result;
use reqwest::Client;

/// A simple implementation of `ClientProvider` that always returns a new `Client`.
///
/// This is useful for basic scenarios where no special client configuration is needed.
///
/// # Example
/// ```rust
/// use core::client_provider::{ClientProvider, StaticClientProvider};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let provider = StaticClientProvider::new();
///     let client = provider.get_client().await?;
///     Ok(())
/// }
/// ```
pub struct StaticClientProvider;

impl StaticClientProvider {
    /// Creates a new `StaticClientProvider`.
    ///
    /// # Example
    /// ```rust
    /// use core::client_provider::StaticClientProvider;
    ///
    /// let provider = StaticClientProvider::new();
    /// ```
    pub fn new() -> Self {
        StaticClientProvider
    }
}

#[async_trait::async_trait]
impl ClientProvider for StaticClientProvider {
    /// Returns a new reqwest HTTP client.
    /// This implementation creates a new client instance each time.
    ///
    /// # Example
    /// ```rust
    /// use core::client_provider::{ClientProvider, StaticClientProvider};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let provider = StaticClientProvider::new();
    ///     let client = provider.get_client().await?;
    ///     Ok(())
    /// }
    /// ```
    async fn get_client(&self) -> Result<Client> {
        Ok(Client::new())
    }
}
