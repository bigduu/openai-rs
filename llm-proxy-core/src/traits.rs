use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;
use std::sync::Arc;

use crate::types::{ResponseStream, Result};

/// Trait for parsing raw request bytes into a structured format.
#[async_trait]
pub trait RequestParser: Send + Sync {
    /// Parse raw request bytes into a JSON Value and detect if streaming was requested.
    /// 
    /// # Arguments
    /// * `body` - The raw request body bytes
    /// 
    /// # Returns
    /// A tuple containing the parsed request as JSON Value and whether streaming was requested
    async fn parse(&self, body: Bytes) -> Result<(Value, bool)>;
}

/// Trait for processing requests before they are sent to the LLM service.
#[async_trait]
pub trait Processor: Send + Sync {
    /// Process a request, potentially modifying it.
    /// 
    /// # Arguments
    /// * `request` - The request to process as JSON Value
    /// 
    /// # Returns
    /// The processed request as JSON Value
    async fn process(&self, request: Value) -> Result<Value>;
}

/// A chain of processors that are executed in sequence.
pub struct ProcessorChain {
    processors: Vec<Arc<dyn Processor>>,
}

impl ProcessorChain {
    /// Create a new processor chain with the given processors.
    pub fn new(processors: Vec<Arc<dyn Processor>>) -> Self {
        Self { processors }
    }

    /// Execute all processors in the chain in sequence.
    pub async fn execute(&self, initial_request: Value) -> Result<Value> {
        let mut request = initial_request;
        for processor in &self.processors {
            request = processor.process(request).await?;
        }
        Ok(request)
    }
}

/// Trait for interacting with an LLM service.
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Execute a request against the LLM service.
    /// 
    /// # Arguments
    /// * `request` - The processed request to send to the LLM
    /// * `stream` - Whether to request a streaming response
    /// 
    /// # Returns
    /// A channel receiver that will receive the raw response bytes
    async fn execute(&self, request: Value, stream: bool) -> Result<ResponseStream>;
}

/// Trait for managing LLM API tokens.
#[async_trait]
pub trait TokenProvider: Send + Sync {
    /// Get an API token for the LLM service.
    async fn get_token(&self) -> Result<String>;
}

/// Trait for providing the LLM service URL.
#[async_trait]
pub trait UrlProvider: Send + Sync {
    /// Get the URL for the LLM service endpoint.
    async fn get_url(&self) -> Result<String>;
}

/// Trait for providing an HTTP client.
#[async_trait]
pub trait ClientProvider: Send + Sync {
    /// Get an HTTP client for making requests.
    async fn get_client(&self) -> Result<reqwest::Client>;
}
