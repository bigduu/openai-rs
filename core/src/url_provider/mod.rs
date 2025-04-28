mod static_url;
pub use static_url::StaticUrlProvider;

use anyhow::Result;
use async_trait::async_trait;

/// Defines the contract for providing API URLs.
///
/// Implementations of this trait are responsible for retrieving the necessary
/// API endpoints required for making requests to downstream LLM APIs.
///
/// This abstraction allows the core forwarding logic to remain decoupled from
/// specific URL configurations. Different strategies (static URLs,
/// dynamic URL generation, environment-based URLs) can be implemented and swapped easily.
#[async_trait]
pub trait UrlProvider: Send + Sync {
    /// Asynchronously retrieves an API URL.
    ///
    /// # Returns
    ///
    /// * `Ok(String)`: Containing the API URL if retrieval is successful.
    /// * `Err(anyhow::Error)`: If an error occurred during URL retrieval.
    async fn get_url(&self) -> Result<String>;
}
