use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{ResponseStream, Result};

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
pub trait LLMRequest: Send + Sync {
    /// Get the messages from the request as a JSON Value.
    /// This typically includes the conversation history or prompt.
    fn messages(&self) -> Result<Value>;

    /// Get the model name or identifier.
    /// This specifies which LLM model should process the request.
    fn model(&self) -> Result<String>;

    /// Get whether streaming is enabled for this request.
    /// When true, the response should be streamed back as it's generated.
    fn stream(&self) -> Result<bool>;

    /// Get the maximum number of tokens to generate.
    /// This is optional and may not be supported by all providers.
    fn max_tokens(&self) -> Option<u32>;

    /// Convert the request to a map of field names to values.
    /// This is useful for providers that need to access all request fields.
    fn to_map(&self) -> Result<HashMap<String, Value>>;

    /// Convert the entire request to a JSON Value.
    /// This is typically used when sending the request to the LLM service.
    fn to_value(&self) -> Result<Value>;
}

/// Trait for response types from LLM services.
///
/// This trait provides methods to convert responses to a format
/// that can be sent back to the client.
pub trait LLMResponse {
    /// Convert the response to bytes that can be sent over the network.
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

/// Trait for processing requests before they are sent to the LLM service.
///
/// Processors can modify requests in various ways, such as:
/// - Enhancing prompts with additional context
/// - Adding system messages
/// - Modifying model parameters
/// - Implementing retry logic
/// - Adding logging or monitoring
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::Processor;
/// #
/// # struct MyLLMRequest;
/// # impl MyLLMRequest {
/// #     fn add_system_message(&mut self, _msg: &str) -> Result<()> { Ok(()) }
/// # }
///
/// struct SystemMessageProcessor {
///     system_message: String,
/// }
///
/// impl SystemMessageProcessor {
///     fn new(msg: &str) -> Self {
///         Self { system_message: msg.to_string() }
///     }
/// }
///
/// #[async_trait]
/// impl Processor<MyLLMRequest> for SystemMessageProcessor {
///     async fn process(&self, mut request: MyLLMRequest) -> Result<MyLLMRequest> {
///         // Add system message to the request
///         request.add_system_message(&self.system_message)?;
///         Ok(request)
///     }
/// }
/// ```
#[async_trait]
pub trait Processor<T: LLMRequest>: Send + Sync {
    /// Process a request, potentially modifying it.
    ///
    /// # Arguments
    /// * `request` - The request to process
    ///
    /// # Returns
    /// The processed request, which may be modified from the input
    async fn process(&self, request: T) -> Result<T>;
}

/// A chain of processors that are executed in sequence.
///
/// This struct allows multiple processors to be combined and executed
/// in order. Each processor's output becomes the input for the next
/// processor in the chain.
///
/// # Example
///
/// ```rust
/// # use std::sync::Arc;
/// # use anyhow::Result;
/// # use llm_proxy_core::ProcessorChain;
/// #
/// # struct MyLLMRequest;
/// # struct SystemMessageProcessor;
/// # impl SystemMessageProcessor {
/// #     fn new(_: &str) -> Self { Self }
/// # }
/// # struct TokenLimitProcessor;
/// # impl TokenLimitProcessor {
/// #     fn new(_: u32) -> Self { Self }
/// # }
/// # struct LoggingProcessor;
/// # impl LoggingProcessor {
/// #     fn new() -> Self { Self }
/// # }
/// #
/// # async fn example() -> Result<()> {
/// let chain = ProcessorChain::new(vec![
///     Arc::new(SystemMessageProcessor::new("Be helpful")),
///     Arc::new(TokenLimitProcessor::new(2000)),
///     Arc::new(LoggingProcessor::new()),
/// ]);
///
/// let request = MyLLMRequest;
/// let processed_request = chain.execute(request).await?;
/// # Ok(())
/// # }
/// ```
pub struct ProcessorChain<T: LLMRequest> {
    processors: Vec<Arc<dyn Processor<T>>>,
}

impl<T: LLMRequest> ProcessorChain<T> {
    /// Create a new processor chain with the given processors.
    /// The processors will be executed in the order they appear in the vector.
    pub fn new(processors: Vec<Arc<dyn Processor<T>>>) -> Self {
        Self { processors }
    }

    /// Execute all processors in the chain in sequence.
    ///
    /// # Arguments
    /// * `initial_request` - The request to process through the chain
    ///
    /// # Returns
    /// The request after being processed by all processors in the chain
    pub async fn execute(&self, initial_request: T) -> Result<T> {
        let mut request = initial_request;
        for processor in &self.processors {
            request = processor.process(request).await?;
        }
        Ok(request)
    }
}

/// Trait for interacting with an LLM service.
///
/// This trait defines the core interface for sending requests to
/// an LLM service and receiving responses. Implementations handle
/// the specifics of communicating with different LLM providers.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::{LLMClient, ResponseStream};
/// #
/// # struct OpenAIRequest;
///
/// struct OpenAIClient {
///     api_key: String,
///     base_url: String,
/// }
///
/// #[async_trait]
/// impl LLMClient<OpenAIRequest> for OpenAIClient {
///     async fn execute(&self, request: OpenAIRequest) -> Result<ResponseStream> {
///         // Send request to OpenAI API and return response stream
///         # todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait LLMClient<T: LLMRequest>: Send + Sync {
    /// Execute a request against the LLM service.
    ///
    /// # Arguments
    /// * `request` - The processed request to send to the LLM
    ///
    /// # Returns
    /// A channel receiver that will receive the response chunks
    async fn execute(&self, request: T) -> Result<ResponseStream>;
}

/// Trait for managing LLM API tokens.
///
/// This trait provides a way to abstract token management,
/// allowing different strategies for token storage and retrieval.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::TokenProvider;
///
/// struct EnvTokenProvider {
///     env_var: String,
/// }
///
/// #[async_trait]
/// impl TokenProvider for EnvTokenProvider {
///     async fn get_token(&self) -> Result<String> {
///         std::env::var(&self.env_var).map_err(Into::into)
///     }
/// }
/// ```
#[async_trait]
pub trait TokenProvider: Send + Sync {
    /// Get an API token for the LLM service.
    /// This might involve reading from environment variables,
    /// secure storage, or a token management service.
    async fn get_token(&self) -> Result<String>;
}

/// Trait for providing the LLM service URL.
///
/// This trait allows the service URL to be configured or
/// determined dynamically at runtime.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use llm_proxy_core::UrlProvider;
///
/// struct ConfigUrlProvider {
///     base_url: String,
/// }
///
/// #[async_trait]
/// impl UrlProvider for ConfigUrlProvider {
///     async fn get_url(&self) -> Result<String> {
///         Ok(self.base_url.clone())
///     }
/// }
/// ```
#[async_trait]
pub trait UrlProvider: Send + Sync {
    /// Get the URL for the LLM service endpoint.
    async fn get_url(&self) -> Result<String>;
}

/// Trait for providing an HTTP client.
///
/// This trait allows the HTTP client to be configured with
/// custom settings or replaced with different implementations.
///
/// # Example
///
/// ```rust
/// # use async_trait::async_trait;
/// # use anyhow::Result;
/// # use std::time::Duration;
/// # use llm_proxy_core::ClientProvider;
///
/// struct CustomClientProvider {
///     timeout: Duration,
/// }
///
/// #[async_trait]
/// impl ClientProvider for CustomClientProvider {
///     async fn get_client(&self) -> Result<reqwest::Client> {
///         reqwest::Client::builder()
///             .timeout(self.timeout)
///             .build()
///             .map_err(Into::into)
///     }
/// }
/// ```
#[async_trait]
pub trait ClientProvider: Send + Sync {
    /// Get an HTTP client for making requests.
    async fn get_client(&self) -> Result<reqwest::Client>;
}
