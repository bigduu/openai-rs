//! Core traits and types for the LLM proxy system.
//!
//! This crate provides the fundamental interfaces and types that form the basis of the LLM proxy.
//! It is intentionally free of specific implementations, serving as a foundation for provider-specific
//! crates to build upon.

pub mod error;
pub mod pipeline;
pub mod traits;
pub mod types;

pub use error::Error;
pub use pipeline::Pipeline;
pub use traits::*;
pub use types::*;
