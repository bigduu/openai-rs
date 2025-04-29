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
///
/// # Example
/// ```rust
/// use core::url_provider::{UrlProvider, StaticUrlProvider};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create a static URL provider
///     let provider: Arc<dyn UrlProvider> = Arc::new(
///         StaticUrlProvider::new("https://api.openai.com/v1/chat/completions".to_string())
///     );
///
///     // Get the URL
///     let url = provider.get_url().await?;
///     assert_eq!(url, "https://api.openai.com/v1/chat/completions");
///     Ok(())
/// }
/// ```
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
