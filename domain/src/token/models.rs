use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents an API token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The actual token string
    pub value: String,
    /// Optional expiration timestamp
    pub expires_at: Option<i64>,
}

/// Provides tokens for API authentication
#[async_trait]
pub trait TokenProvider: Send + Sync {
    /// Retrieves a valid token
    async fn get_token(&self) -> Result<Token>;

    /// Validates if a token is still valid
    async fn is_valid(&self, token: &Token) -> bool {
        if let Some(expires_at) = token.expires_at {
            use chrono::Utc;
            Utc::now().timestamp() < expires_at
        } else {
            true
        }
    }
}
