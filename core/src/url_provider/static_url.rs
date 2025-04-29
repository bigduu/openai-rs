use super::UrlProvider;
use anyhow::Result;
use async_trait::async_trait;

/// A simple `UrlProvider` that always returns a fixed, pre-configured URL.
/// Useful for scenarios where the API endpoint does not change, like
/// standard OpenAI API endpoints.
///
/// # Example
/// ```rust
/// use core::url_provider::{UrlProvider, StaticUrlProvider};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create a new static URL provider
///     let provider = StaticUrlProvider::new("https://api.example.com".to_string());
///
///     // Get the URL
///     let url = provider.get_url().await?;
///     assert_eq!(url, "https://api.example.com");
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct StaticUrlProvider {
    url: String,
}

impl StaticUrlProvider {
    /// Creates a new `StaticUrlProvider`.
    ///
    /// # Arguments
    ///
    /// * `url`: The static URL string to be returned by `get_url`.
    ///
    /// # Example
    /// ```rust
    /// use core::url_provider::StaticUrlProvider;
    ///
    /// let provider = StaticUrlProvider::new("https://api.openai.com".to_string());
    /// ```
    pub fn new(url: String) -> Self {
        StaticUrlProvider { url }
    }
}

#[async_trait]
impl UrlProvider for StaticUrlProvider {
    /// Returns the pre-configured static URL.
    /// This implementation does not involve any I/O or complex logic,
    /// so it returns immediately with the stored URL.
    ///
    /// # Example
    /// ```rust
    /// use core::url_provider::{UrlProvider, StaticUrlProvider};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let provider = StaticUrlProvider::new("https://api.test.com".to_string());
    ///     let url = provider.get_url().await?;
    ///     assert_eq!(url, "https://api.test.com");
    ///     Ok(())
    /// }
    /// ```
    async fn get_url(&self) -> Result<String> {
        Ok(self.url.clone())
    }
}
