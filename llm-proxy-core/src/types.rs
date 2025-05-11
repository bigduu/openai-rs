use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::Error;

/// The result type used throughout the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Represents a raw response stream from an LLM service
pub type ResponseStream = mpsc::Receiver<Result<Bytes>>;

/// Configuration for a specific LLM backend service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// The type of LLM provider (e.g., "openai", "anthropic", etc.)
    pub provider: String,
    /// The type of endpoint (e.g., "chat", "completion", "embedding")
    pub endpoint_type: String,
    /// Base URL for the LLM API
    pub base_url: String,
    /// Whether this endpoint supports streaming responses
    pub supports_streaming: bool,
    /// Environment variable name containing the API key
    pub token_env: String,
    /// Additional provider-specific configuration (JSON object)
    #[serde(default)]
    pub additional_config: serde_json::Value,
}

/// Configuration for a processor in the processing chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// The type of processor (e.g., `api_call`, `simple_logger`, etc.)
    pub processor_type: String,
    /// Configuration value for the processor
    pub config_value: String,
    /// Additional processor-specific configuration (JSON object)
    #[serde(default)]
    pub additional_config: serde_json::Value,
}
