use super::ClientProvider;
use anyhow::Result;
use reqwest::Client;

/// A simple implementation of `ClientProvider` that always returns a new `Client`.
///
/// This is useful for basic scenarios where no special client configuration is needed.
pub struct StaticClientProvider;

impl StaticClientProvider {
    pub fn new() -> Self {
        StaticClientProvider
    }
}

#[async_trait::async_trait]
impl ClientProvider for StaticClientProvider {
    async fn get_client(&self) -> Result<Client> {
        Ok(Client::new())
    }
}
