//! Module responsible for managing and executing the chain of processors.

use crate::openai_types::chat::OpenAiChatCompletionRequest;
use crate::processor::Processor;
use anyhow::Result;

/// A chain of processors that can be executed in sequence.
///
/// The `ProcessorChain` allows multiple processors to be combined and executed
/// in a specific order. Each processor in the chain can modify the messages
/// before passing them to the next processor.
///
/// # Example
/// ```rust
/// use core::processor_chain::ProcessorChain;
/// use core::processor::Processor;
/// use core::openai_types::chat::{OpenAiChatCompletionRequest, OpenAiChatMessage};
/// use std::sync::Arc;
/// use std::option::Option;
/// use async_trait::async_trait;
///
/// // Example processors
/// struct PrefixProcessor {
///     prefix: String,
/// }
///
/// #[async_trait::async_trait]
/// impl Processor for PrefixProcessor {
///     async fn process_messages(
///         &self,
///         messages: Vec<OpenAiChatMessage>,
///     ) -> anyhow::Result<Vec<OpenAiChatMessage>> {
///         Ok(messages.into_iter().map(|mut msg| {
///             if let Some(content) = msg.content {
///                 msg.content = Some(format!("{} {}", self.prefix, content));
///             }
///             msg
///         }).collect())
///     }
/// }
///
/// struct SuffixProcessor {
///     suffix: String,
/// }
///
/// #[async_trait::async_trait]
/// impl Processor for SuffixProcessor {
///     async fn process_messages(
///         &self,
///         messages: Vec<OpenAiChatMessage>,
///     ) -> anyhow::Result<Vec<OpenAiChatMessage>> {
///         Ok(messages.into_iter().map(|mut msg| {
///             if let Some(content) = msg.content {
///                 msg.content = Some(format!("{} {}", content, self.suffix));
///             }
///             msg
///         }).collect())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create processors
///     let prefix_processor = Box::new(PrefixProcessor {
///         prefix: "[Prefix]".to_string(),
///     });
///     let suffix_processor = Box::new(SuffixProcessor {
///         suffix: "[Suffix]".to_string(),
///     });
///
///     // Create processor chain
///     let chain = ProcessorChain::new(vec![prefix_processor, suffix_processor]);
///
///     // Create a request
///     let mut request = OpenAiChatCompletionRequest {
///         model: "gpt-3.5-turbo".to_string(),
///         messages: vec![
///             OpenAiChatMessage {
///                 role: "user".to_string(),
///                 content: Some("Hello".to_string()),
///                 name: None,
///                 tool_call_id: None,
///                 tool_calls: None,
///             }
///         ],
///         stream: None,
///         frequency_penalty: None,
///         logit_bias: None,
///         logprobs: None,
///         top_logprobs: None,
///         max_tokens: None,
///         n: None,
///         presence_penalty: None,
///         response_format: None,
///         seed: None,
///         stop: None,
///         temperature: None,
///         top_p: None,
///         tools: None,
///         tool_choice: None,
///         user: None,
///     };
///
///     // Execute the chain
///     let processed = chain.execute(request).await?;
///     assert_eq!(processed.messages[0].content, Some("[Prefix] Hello [Suffix]".to_string()));
///     Ok(())
/// }
/// ```
pub struct ProcessorChain {
    processors: Vec<Box<dyn Processor>>,
}

impl ProcessorChain {
    /// Creates a new processor chain with the given processors.
    ///
    /// # Arguments
    ///
    /// * `processors`: A vector of processors to be executed in sequence.
    ///
    /// # Example
    /// ```rust
    /// use core::processor_chain::ProcessorChain;
    /// use core::processor::Processor;
    ///
    /// struct DummyProcessor;
    ///
    /// #[async_trait::async_trait]
    /// impl Processor for DummyProcessor {
    ///     async fn process_messages(
    ///         &self,
    ///         messages: Vec<OpenAiChatMessage>,
    ///     ) -> anyhow::Result<Vec<OpenAiChatMessage>> {
    ///         Ok(messages)
    ///     }
    /// }
    ///
    /// let chain = ProcessorChain::new(vec![Box::new(DummyProcessor)]);
    /// ```
    pub fn new(processors: Vec<Box<dyn Processor>>) -> Self {
        ProcessorChain { processors }
    }

    /// Executes the processor chain on the given request.
    ///
    /// # Arguments
    ///
    /// * `request`: The request to be processed.
    ///
    /// # Returns
    ///
    /// The processed request after all processors have been applied.
    ///
    /// # Example
    /// ```rust
    /// use core::processor_chain::ProcessorChain;
    /// use core::openai_types::chat::{OpenAiChatCompletionRequest, OpenAiChatMessage};
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let chain = ProcessorChain::new(vec![]);
    ///     let request = OpenAiChatCompletionRequest {
    ///         model: "gpt-3.5-turbo".to_string(),
    ///         messages: vec![
    ///             OpenAiChatMessage {
    ///                 role: "user".to_string(),
    ///                 content: Some("Hello".to_string()),
    ///                 name: None,
    ///                 tool_call_id: None,
    ///                 tool_calls: None,
    ///             }
    ///         ],
    ///         stream: None,
    ///         frequency_penalty: None,
    ///         logit_bias: None,
    ///         logprobs: None,
    ///         top_logprobs: None,
    ///         max_tokens: None,
    ///         n: None,
    ///         presence_penalty: None,
    ///         response_format: None,
    ///         seed: None,
    ///         stop: None,
    ///         temperature: None,
    ///         top_p: None,
    ///         tools: None,
    ///         tool_choice: None,
    ///         user: None,
    ///     };
    ///
    ///     let processed = chain.execute(request).await?;
    ///     assert_eq!(processed.messages[0].content, Some("Hello".to_string()));
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute(
        &self,
        mut request: OpenAiChatCompletionRequest,
    ) -> Result<OpenAiChatCompletionRequest> {
        let mut messages = request.messages;

        for processor in &self.processors {
            messages = processor.process_messages(messages).await?;
        }

        request.messages = messages;
        Ok(request)
    }
}
