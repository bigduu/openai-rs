use std::sync::Arc;

use async_trait::async_trait;

use crate::types::Result;

use crate::LLMRequest;

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
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// This function will return an error if the request processing fails.
    pub async fn execute(&self, initial_request: T) -> Result<T> {
        let mut request = initial_request;
        for processor in &self.processors {
            request = processor.process(request).await?;
        }
        Ok(request)
    }
}
