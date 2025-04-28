use super::UrlProvider;
use anyhow::Result;
use async_trait::async_trait;

/// A simple `UrlProvider` that always returns a fixed, pre-configured URL.
/// Useful for scenarios where the API endpoint does not change, like
/// standard OpenAI API endpoints.
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
    pub fn new(url: String) -> Self {
        StaticUrlProvider { url }
    }
}

#[async_trait]
impl UrlProvider for StaticUrlProvider {
    /// Returns the pre-configured static URL.
    /// This implementation does not involve any I/O or complex logic,
    /// so it returns immediately with the stored URL.
    async fn get_url(&self) -> Result<String> {
        Ok(self.url.clone())
    }
}
