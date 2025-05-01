//! # LLM Proxy Core
//!
//! This crate provides the core abstractions and traits for the LLM Proxy system.
//! It defines the fundamental interfaces that all LLM providers and request processors
//! must implement.
//!
//! ## Core Components
//!
//! ### Pipeline
//! The [`Pipeline`] struct is the central component that manages the flow of requests
//! through processors and to the LLM provider. It handles both streaming and non-streaming
//! responses in a unified way.
//!
//! ### Provider Trait
//! The [`Provider`] trait defines the interface that all LLM providers must implement.
//! This includes methods for processing requests and handling responses.
//!
//! ### Processor Trait
//! The [`Processor`] trait defines the interface for request processors that can modify
//! or enhance requests before they reach the LLM provider.
//!
//! ### Types
//! Common types used throughout the system are defined in the [`types`] module.
//! This includes configuration types, request/response types, and other shared structures.
//!
//! ### Error Handling
//! The [`error`] module provides a unified error handling system used across all
//! LLM proxy components.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use llm_proxy_core::{Pipeline, Provider, Processor};
//!
//! // Create a pipeline with a provider and processors
//! let pipeline = Pipeline::new(
//!     provider,
//!     vec![processor1, processor2],
//!     route_config
//! );
//!
//! // Process a request through the pipeline
//! let response = pipeline.execute(request).await?;
//! ```

pub mod error;
pub mod pipeline;
pub mod traits;
pub mod types;

pub use error::Error;
pub use pipeline::Pipeline;
pub use traits::*;
pub use types::*;
