use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;

/// Server configuration loaded from config.toml
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// LLM backend configurations
    pub llm: HashMap<String, LLMConfig>,
    /// Processor configurations
    pub processor: HashMap<String, ProcessorConfig>,
    /// Route configurations
    pub route: Vec<RouteConfig>,
    /// Server-specific settings
    pub server: ServerConfig,
}

/// Configuration for an LLM backend service
#[derive(Debug, Deserialize, Clone)]
pub struct LLMConfig {
    /// The type of LLM provider (e.g., "openai", "anthropic")
    pub provider: String,
    /// The type of endpoint (e.g., "chat", "completion", "embedding")
    #[serde(rename = "type")]
    pub endpoint_type: String,
    /// Base URL for the LLM API
    pub base_url: String,
    /// Environment variable containing the API token
    pub token_env: String,
    /// Whether this endpoint supports streaming responses
    pub supports_streaming: bool,
    /// Additional provider-specific configuration
    #[serde(default)]
    pub additional_config: serde_json::Value,
}

/// Configuration for a processor in the processing chain
#[derive(Debug, Deserialize, Clone)]
pub struct ProcessorConfig {
    /// The type of processor
    #[serde(rename = "type")]
    pub processor_type: String,
    /// Primary configuration value
    pub config_value: String,
    /// Additional processor-specific configuration
    #[serde(default)]
    pub additional_config: serde_json::Value,
}

/// Configuration for a route that maps requests to LLM backends
#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    /// The URL path prefix to match for this route
    pub path_prefix: String,
    /// The ID of the target LLM backend to use
    pub target_llm: String,
    /// List of processor IDs to apply in order
    pub processors: Vec<String>,
    /// Whether to allow streaming responses on this route
    pub allow_streaming: bool,
    /// Whether to allow non-streaming responses on this route
    pub allow_non_streaming: bool,
}

/// Server-specific configuration settings
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: IpAddr,
    /// Port to listen on
    pub port: u16,
    /// Logging level (ERROR, WARN, INFO, DEBUG, TRACE)
    pub log_level: String,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;

        config.try_deserialize().map_err(|e| anyhow::anyhow!(e))
    }

    /// Get a route configuration that matches the given path
    pub fn find_route(&self, path: &str) -> Option<&RouteConfig> {
        self.route
            .iter()
            .find(|route| path.starts_with(&route.path_prefix))
    }

    /// Get an LLM configuration by ID
    pub fn get_llm(&self, id: &str) -> anyhow::Result<&LLMConfig> {
        self.llm
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("LLM configuration not found for ID: {id}"))
    }

    /// Get a processor configuration by ID
    pub fn get_processor(&self, id: &str) -> anyhow::Result<&ProcessorConfig> {
        self.processor
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Processor configuration not found for ID: {id}"))
    }
}
