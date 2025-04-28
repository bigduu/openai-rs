//! Core library for the Rust Intelligent Streaming Proxy Enhancement System.
//!
//! This crate defines the fundamental types, traits, and structures used
//! throughout the system, including the internal event format, processor
//! and token provider interfaces, and potentially common utilities.

// Module declarations
pub mod client_provider;
pub mod openai_types;
pub mod processor;
pub mod processor_chain;
pub mod token_provider;
pub mod url_provider;

// Placeholder modules for future implementation
// pub mod http; // Likely handled in main.rs or a separate crate if it grows complex
pub mod forwarder;
pub mod parser;

// Re-export key types for easier access
pub use processor::Processor;
pub use processor_chain::ProcessorChain;
pub use token_provider::{StaticTokenProvider, TokenProvider}; // Added StaticTokenProvider
pub use url_provider::{StaticUrlProvider, UrlProvider};
