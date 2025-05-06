//! # LLM Proxy Core
//!
//! This crate provides the core abstractions and traits for building a proxy server for Large Language Model APIs.
//! It enables building flexible, configurable, and extensible proxy servers that can work with multiple LLM providers
//! while providing enhanced functionality like request processing pipelines and unified interfaces.
//!
//! ## Architecture
//!
//! The system is built around three main components:
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌────────────┐
//! │   Request   │ ──> │  Processing  │ ──> │    LLM     │
//! │   Parser    │     │   Pipeline   │     │   Client   │
//! └─────────────┘     └──────────────┘     └────────────┘
//! ```
//!
//! ## Core Components
//!
//! ### Pipeline
//! The [`Pipeline`] struct orchestrates the flow of requests through the system:
//! - Parses incoming raw requests into structured data
//! - Routes requests through configurable processing chains
//! - Handles communication with LLM providers
//! - Manages both streaming and non-streaming responses
//!
//! ### Request Processing
//! - [`RequestParser`]: Converts raw requests into structured types
//! - [`Processor`]: Transforms and enhances requests before they reach the provider
//! - [`ProcessorChain`]: Combines multiple processors into a sequential pipeline
//!
//! ### Provider Integration
//! - [`LLMClient`]: Handles communication with specific LLM providers
//! - [`LLMRequest`]: Defines the interface for structured requests
//!
//! ### Supporting Components
//! - [`TokenProvider`]: Manages API tokens and authentication
//! - [`UrlProvider`]: Provides service endpoints
//! - [`ClientProvider`]: Configures HTTP clients
//!
//! ## Example Usage
//!
//! ```rust
//! # use std::sync::Arc;
//! # use anyhow::Result;
//! # use bytes::Bytes;
//! # use async_trait::async_trait;
//! # use llm_proxy_core::{Pipeline, Processor, LLMClient, RequestParser, ResponseStream, LLMRequest};
//! # use tokio::sync::mpsc;
//! #
//! # struct MyRequest;
//! # impl LLMRequest for MyRequest {
//! #     fn messages(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
//! #     fn model(&self) -> Result<String> { Ok("model".to_string()) }
//! #     fn stream(&self) -> Result<bool> { Ok(false) }
//! #     fn max_tokens(&self) -> Option<u32> { None }
//! #     fn to_map(&self) -> Result<std::collections::HashMap<String, serde_json::Value>> { Ok(std::collections::HashMap::new()) }
//! #     fn to_value(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
//! # }
//! #
//! # struct MyRequestParser;
//! # #[async_trait]
//! # impl RequestParser<MyRequest> for MyRequestParser {
//! #     async fn parse(&self, _body: Bytes) -> Result<MyRequest> {
//! #         Ok(MyRequest)
//! #     }
//! # }
//! #
//! # struct MyProcessor;
//! # #[async_trait]
//! # impl Processor<MyRequest> for MyProcessor {
//! #     async fn process(&self, request: MyRequest) -> Result<MyRequest> {
//! #         Ok(request)
//! #     }
//! # }
//! #
//! # struct MyLLMClient;
//! # #[async_trait]
//! # impl LLMClient<MyRequest> for MyLLMClient {
//! #     async fn execute(&self, _request: MyRequest) -> Result<ResponseStream> {
//! #         let (tx, rx) = mpsc::channel(1);
//! #         Ok(rx)
//! #     }
//! # }
//! #
//! # async fn example() -> Result<()> {
//! // Create pipeline components
//! let parser = Arc::new(MyRequestParser);
//! let processors = vec![Arc::new(MyProcessor)];
//! let processor_chain = Arc::new(llm_proxy_core::ProcessorChain::new(processors));
//! let llm_client = Arc::new(MyLLMClient);
//!
//! // Create and configure pipeline
//! let pipeline = Pipeline::new(parser, processor_chain, llm_client);
//!
//! // Process request
//! let request = Bytes::from("{}");
//! let response_stream = pipeline.execute(request).await?;
//!
//! // Handle response stream
//! while let Some(chunk) = response_stream.recv().await {
//!     match chunk {
//!         Ok(data) => println!("Received chunk: {:?}", data),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod pipeline;
pub mod traits;
pub mod types;

pub use error::Error;
pub use pipeline::Pipeline;
pub use traits::{
    client::ClientProvider, client::LLMClient, client::TokenProvider, client::UrlProvider,
    processor::Processor, processor::ProcessorChain, request::LLMRequest, request::LLMResponse,
    request::RequestParser,
};
pub use types::*;

// Re-export Result from types module
pub use types::Result;
