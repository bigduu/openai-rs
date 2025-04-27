use anyhow::Result;
use domain::token::{Token, TokenProvider};
use std::{collections::HashMap, sync::Arc};

/// Manages multiple token providers and their configurations
pub struct TokenVault {
    providers: HashMap<String, Arc<dyn TokenProvider>>,
}

impl TokenVault {
    /// Creates a new empty TokenVault
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Adds a token provider for a specific model or endpoint
    pub fn add_provider(&mut self, key: String, provider: Arc<dyn TokenProvider>) {
        self.providers.insert(key, provider);
    }

    /// Retrieves a token for a specific model or endpoint
    pub async fn get_token(&self, key: &str) -> Result<Token> {
        if let Some(provider) = self.providers.get(key) {
            provider.get_token().await
        } else {
            anyhow::bail!("No token provider found for key: {}", key)
        }
    }

    /// Validates if a token is still valid
    pub async fn is_valid(&self, key: &str, token: &Token) -> Result<bool> {
        if let Some(provider) = self.providers.get(key) {
            Ok(provider.is_valid(token).await)
        } else {
            anyhow::bail!("No token provider found for key: {}", key)
        }
    }
}

impl Default for TokenVault {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple token provider that always returns a static token
pub struct StaticTokenProvider {
    token: String,
}

impl StaticTokenProvider {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[async_trait::async_trait]
impl TokenProvider for StaticTokenProvider {
    async fn get_token(&self) -> Result<Token> {
        Ok(Token {
            value: self.token.clone(),
            expires_at: None,
        })
    }
}
