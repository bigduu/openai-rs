//! # LLM Proxy Server
//!
//! This crate provides the HTTP server implementation for the LLM Proxy system.
//! It handles request routing, pipeline management, and server configuration.
//!
//! ## Features
//!
//! - HTTP server with configurable routing
//! - CORS support and security features
//! - Dynamic pipeline management
//! - Configuration through TOML files
//! - Comprehensive logging and error handling
//!
//! ## Components
//!
//! ### App
//! The [`app`] module contains the core server implementation, including:
//! - HTTP server setup and configuration
//! - Request handling and routing
//! - Pipeline registry management
//!
//! ### Config
//! The [`config`] module handles server configuration, including:
//! - TOML file parsing
//! - Route configuration
//! - LLM provider settings
//! - Server settings (host, port, CORS)
//!
//! ## Server Configuration
//!
//! The server is configured through a TOML file with the following sections:
//!
//! ```toml
//! [server]
//! host = "127.0.0.1"
//! port = 3000
//! cors_allowed_origins = ["*"]
//!
//! [llm.provider_name]
//! provider = "openai"
//! type = "chat"
//! base_url = "https://api.openai.com/v1"
//! token_env = "OPENAI_API_KEY"
//!
//! [[route]]
//! path_prefix = "/v1/chat/completions"
//! target_llm = "provider_name"
//! allow_streaming = true
//! ```
//!
//! ## Usage
//!
//! To start the server:
//!
//! ```rust,no_run
//! use llm_proxy_server::{config, app};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = config::load_config("config.toml")?;
//!     
//!     // Start the server
//!     app::run_server(config).await
//! }
//! ```
//!
//! ## Error Handling
//!
//! The server provides comprehensive error handling and logging:
//! - Request validation errors
//! - Configuration errors
//! - Pipeline execution errors
//! - Provider-specific errors
//!
//! All errors are properly logged and appropriate HTTP status codes are returned.

pub mod app;
pub mod config;

pub use app::run_server;
pub use config::Config;
