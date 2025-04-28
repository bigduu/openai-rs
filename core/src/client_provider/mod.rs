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
