mod static_client;
pub use static_client::StaticClientProvider;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

/// Defines the contract for providing HTTP clients.
///
/// Implementations of this trait are responsible for creating and managing
/// HTTP clients used for making requests to downstream LLM APIs.
///
/// This abstraction allows the core forwarding logic to remain decoupled from
/// specific client configurations and implementations.
///
/// # Example
/// ```rust
/// use core::client_provider::{ClientProvider, StaticClientProvider};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create a static client provider
///     let provider: Arc<dyn ClientProvider> = Arc::new(StaticClientProvider::new());
///
///     // Get a client
///     let client = provider.get_client().await?;
///     
///     // The client can now be used to make HTTP requests
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait ClientProvider: Send + Sync {
    /// Asynchronously retrieves an HTTP client.
    ///
    /// # Returns
    ///
    /// * `Ok(Client)`: Containing the HTTP client if retrieval is successful.
    /// * `Err(anyhow::Error)`: If an error occurred during client creation.
    async fn get_client(&self) -> Result<Client>;
}
