use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{ResponseStream, Result};

/// Trait for abstracting LLM request types
pub trait LLMRequest: Send + Sync {
    /// Get the messages from the request
    fn messages(&self) -> Result<Value>;

    /// Get the model name
    fn model(&self) -> Result<String>;

    /// Get whether streaming is enabled
    fn stream(&self) -> Result<bool>;

    /// Get the maximum number of tokens
    fn max_tokens(&self) -> Option<u32>;

    /// Get all fields as a HashMap
    fn to_map(&self) -> Result<HashMap<String, Value>>;

    /// Convert the request to JSON Value
    fn to_value(&self) -> Result<Value>;
}

/// Trait for abstracting LLM response types
pub trait LLMResponse {
    /// Convert the response to bytes
    fn to_bytes(&self) -> Result<Bytes>;
}

/// Trait for parsing raw request bytes into a structured format.
#[async_trait]
pub trait RequestParser<T: LLMRequest>: Send + Sync {
    /// Parse raw request bytes into a specific LLMRequest implementation.
    ///
    /// # Arguments
    /// * `body` - The raw request body bytes
    ///
    /// # Returns
    /// The parsed request as the specific LLMRequest implementation
    async fn parse(&self, body: Bytes) -> Result<T>;
}

/// Trait for processing requests before they are sent to the LLM service.
#[async_trait]
pub trait Processor<T: LLMRequest>: Send + Sync {
    /// Process a request, potentially modifying it.
    ///
    /// # Arguments
    /// * `request` - The request to process as JSON Value
    ///
    /// # Returns
    /// The processed request as JSON Value
    async fn process(&self, request: T) -> Result<T>;
}

/// A chain of processors that are executed in sequence.
pub struct ProcessorChain<T: LLMRequest> {
    processors: Vec<Arc<dyn Processor<T>>>,
}

impl<T: LLMRequest> ProcessorChain<T> {
    /// Create a new processor chain with the given processors.
    pub fn new(processors: Vec<Arc<dyn Processor<T>>>) -> Self {
        Self { processors }
    }

    /// Execute all processors in the chain in sequence.
    pub async fn execute(&self, initial_request: T) -> Result<T> {
        let mut request = initial_request;
        for processor in &self.processors {
            request = processor.process(request).await?;
        }
        Ok(request)
    }
}

/// Trait for interacting with an LLM service.
#[async_trait]
pub trait LLMClient<T: LLMRequest>: Send + Sync {
    /// Execute a request against the LLM service.
    ///
    /// # Arguments
    /// * `request` - The processed request to send to the LLM
    /// * `stream` - Whether to request a streaming response
    ///
    /// # Returns
    /// A channel receiver that will receive the raw response bytes
    async fn execute(&self, request: T) -> Result<ResponseStream>;
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
