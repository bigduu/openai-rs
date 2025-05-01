//! # LLM Proxy OpenAI
//!
//! This crate provides OpenAI-specific implementations for the LLM Proxy system.
//! It implements the core traits defined in `llm-proxy-core` for OpenAI's Chat API.
//!
//! ## Features
//!
//! - Full support for OpenAI's Chat Completions API
//! - Streaming and non-streaming response handling
//! - Efficient request processing and response parsing
//! - Automatic token management and API key handling
//! - Configurable base URL for API endpoints
//!
//! ## Components
//!
//! ### Client
//! The [`client`] module provides a high-level client for interacting with OpenAI's API.
//! It handles authentication, request formatting, and response parsing.
//!
//! ### Providers
//! The [`providers`] module implements the `Provider` trait from `llm-proxy-core`
//! for OpenAI's services. This includes handling both streaming and non-streaming
//! chat completions.
//!
//! ### Types
//! The [`types`] module defines OpenAI-specific types for requests and responses,
//! including chat messages, model parameters, and API responses.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use llm_proxy_openai::{create_chat_pipeline, ChatCompletionRequest};
//!
//! // Create a pipeline for OpenAI chat completions
//! let pipeline = create_chat_pipeline(
//!     processors,
//!     Some("OPENAI_API_KEY"),
//!     Some("https://api.openai.com/v1"),
//!     Some(route_config)
//! );
//!
//! // Process a chat completion request
//! let response = pipeline.execute(request).await?;
//! ```
//!
//! ## Configuration
//!
//! The OpenAI implementation can be configured through environment variables
//! and the route configuration:
//!
//! ```toml
//! [llm.openai_chat]
//! provider = "openai"
//! type = "chat"
//! base_url = "https://api.openai.com/v1"
//! token_env = "OPENAI_API_KEY"
//! supports_streaming = true
//! ```

pub mod client;
pub mod providers;
pub mod types;

use std::sync::Arc;

use llm_proxy_core::{traits::ProcessorChain, Pipeline};

pub use client::OpenAIClient;
pub use providers::{EnvTokenProvider, OpenAIRequestParser, OpenAIUrlProvider};
use providers::{StaticClientProvider, StaticTokenProvider};
pub use types::*;

use llm_proxy_core::Processor;

/// Create a new pipeline configured for OpenAI's chat completion API.
///
/// # Arguments
/// * `processors` - Optional list of processors to apply to requests
/// * `token_env_var` - Environment variable containing the OpenAI API key (default: "OPENAI_API_KEY")
/// * `base_url` - Optional base URL for the API (default: "https://api.openai.com/v1/chat/completions")
/// * `route_config` - Optional route configuration for the RequestParser
///
/// # Returns
/// A pipeline configured with OpenAI-specific components
///
/// # Example
/// ```rust
/// use llm_proxy_openai::create_chat_pipeline;
/// use llm_proxy_core::traits::Processor;
/// use std::sync::Arc;
///
/// // Create a pipeline with no processors
/// let simple_pipeline = create_chat_pipeline(vec![], None, None, None);
///
/// // Create a pipeline with custom processors and API key env var
/// let processors = vec![
///     Arc::new(my_custom_processor::new())
/// ];
/// let pipeline = create_chat_pipeline(
///     processors,
///     Some("MY_OPENAI_KEY"),
///     None,
///     None
/// );
/// ```
pub fn create_chat_pipeline(
    processors: Vec<Arc<dyn Processor<ChatCompletionRequest>>>,
    token_env_var: Option<&str>,
    base_url: Option<&str>,
    route_config: Option<llm_proxy_core::types::RouteConfig>,
) -> Pipeline<ChatCompletionRequest> {
    let client_provider = Arc::new(StaticClientProvider::new());
    let token_provider = Arc::new(StaticTokenProvider::new(token_env_var.unwrap_or("")));
    let url_provider = Arc::new(OpenAIUrlProvider::new(
        base_url.unwrap_or("https://api.openai.com/v1/chat/completions"),
    ));
    let parser = Arc::new(OpenAIRequestParser::new(route_config));
    let processor_chain = Arc::new(ProcessorChain::new(processors));
    let llm_client = Arc::new(OpenAIClient::new(
        client_provider.clone(),
        token_provider,
        url_provider,
    ));

    Pipeline::new(parser, processor_chain, llm_client)
}
