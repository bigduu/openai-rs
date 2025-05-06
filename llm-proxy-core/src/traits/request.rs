use std::collections::HashMap;

use crate::types::Result;
use async_trait::async_trait;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Core trait that defines the interface for LLM requests.
///
/// This trait should be implemented by any struct that represents
/// a request to an LLM service. It provides methods to access common
/// request properties and convert the request to different formats.
///
/// # Example
///
/// ```rust
/// # use serde_json::Value;
/// # use std::collections::HashMap;
/// # use anyhow::Result;
/// #
/// struct Message {
///     content: String,
/// }
///
/// struct MyLLMRequest {
///     messages: Vec<Message>,
///     model: String,
///     stream: bool,
/// }
///
/// # use llm_proxy_core::LLMRequest;
/// impl LLMRequest for MyLLMRequest {
///     fn messages(&self) -> Result<Value> {
///         serde_json::to_value(&self.messages).map_err(Into::into)
///     }
///     
///     fn model(&self) -> Result<String> {
///         Ok(self.model.clone())
///     }
///     
///     fn stream(&self) -> Result<bool> {
///         Ok(self.stream)
///     }
///     
///     fn max_tokens(&self) -> Option<u32> {
///         None
///     }
///     
///     fn to_map(&self) -> Result<HashMap<String, Value>> {
///         Ok(HashMap::new())
///     }
///     
///     fn to_value(&self) -> Result<Value> {
///         Ok(Value::Null)
///     }
/// }
/// ```
pub trait LLMRequest: Send + Sync + DeserializeOwned {
    /// Get the messages from the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the messages as a `Value` if successful, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the messages cannot be retrieved.
    #[allow(clippy::missing_errors_doc)]
    fn messages(&self) -> Result<Value>;

    /// Get the model from the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the model name as a `String` if successful, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the model cannot be retrieved.
    #[allow(clippy::missing_errors_doc)]
    fn model(&self) -> Result<String>;

    /// Check if the request is a streaming request.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating if the request is a streaming request, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the streaming status cannot be determined.
    #[allow(clippy::missing_errors_doc)]
    fn stream(&self) -> Result<bool>;

    /// Get the maximum number of tokens to generate.
    /// This is optional and may not be supported by all providers.
    fn max_tokens(&self) -> Option<u32>;

    /// Convert the request to a map.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` of the request data if successful, or an error if the conversion fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request cannot be converted to a map.
    #[allow(clippy::missing_errors_doc)]
    fn to_map(&self) -> Result<HashMap<String, Value>>;

    /// Convert the request to a JSON value.
    ///
    /// # Returns
    ///
    /// A `Result` containing the request as a `Value` if successful, or an error if the conversion fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request cannot be converted to a JSON value.
    #[allow(clippy::missing_errors_doc)]
    fn to_value(&self) -> Result<Value>;

    /// Convert the request to bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the request as `Bytes` if successful, or an error if the conversion fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request cannot be converted to bytes.
    #[allow(clippy::missing_errors_doc)]
    fn to_bytes(&self) -> Result<Bytes>;
}

/// Trait for response types from LLM services.
///
/// This trait provides methods to convert responses to a format
/// that can be sent back to the client.
pub trait LLMResponse {
    /// Convert the response to bytes that can be sent over the network.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response as `Bytes` if successful, or an error if the conversion fails.
    ///
    /// # Errors
    fn to_bytes(&self) -> Result<Bytes>;
}

/// Trait for parsing raw request bytes into structured requests.
///
/// This trait is responsible for converting the raw bytes received
/// from clients into properly structured request objects.
///
/// # Example
///
/// ```rust
/// # use bytes::Bytes;
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::RequestParser;
/// #
/// # struct MyLLMRequest;
/// # impl MyLLMRequest {
/// #     fn new() -> Self { Self }
/// # }
///
/// struct MyRequestParser;
///
/// #[async_trait]
/// impl RequestParser<MyLLMRequest> for MyRequestParser {
///     async fn parse(&self, body: Bytes) -> Result<MyLLMRequest> {
///         // Parse the bytes into your request type
///         Ok(MyLLMRequest::new())
///     }
/// }
/// ```
#[async_trait]
pub trait RequestParser<T: LLMRequest>: Send + Sync {
    /// Parse raw request bytes into a specific LLMRequest implementation.
    async fn parse(&self, body: Bytes) -> Result<T>;
}
