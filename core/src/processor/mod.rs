use crate::openai_types::chat::OpenAiChatMessage;
use crate::openai_types::function::ToolCall;
use anyhow::Result;
use async_trait::async_trait;
use std::option::Option;

/// Defines the contract for a processor in the stream processing chain.
///
/// Processors are responsible for inspecting, modifying, filtering, or generating
/// messages as they flow through the pipeline.
/// They can perform tasks like content filtering, data enrichment (e.g., RAG), formatting,
/// or triggering side effects based on the message content.
///
/// Processors operate asynchronously, allowing them to perform I/O operations
/// (like calling external APIs or databases) without blocking the entire stream.
///
/// # Example
/// ```rust
/// use core::processor::Processor;
/// use core::openai_types::chat::OpenAiChatMessage;
/// use std::sync::Arc;
/// use std::option::Option;
/// use async_trait::async_trait;
///
/// // Example processor that adds a prefix to all messages
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
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let processor: Arc<dyn Processor> = Arc::new(PrefixProcessor {
///         prefix: "[Processed]".to_string(),
///     });
///
///     let messages = vec![
///         OpenAiChatMessage {
///             role: "user".to_string(),
///             content: Some("Hello".to_string()),
///             name: None,
///             tool_call_id: None,
///             tool_calls: None,
///         }
///     ];
///
///     let processed = processor.process_messages(messages).await?;
///     assert_eq!(processed[0].content, Some("[Processed] Hello".to_string()));
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait Processor: Send + Sync {
    /// Processes a list of messages.
    ///
    /// # Arguments
    ///
    /// * `messages`: A vector of messages to be processed.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<OpenAiChatMessage>)`: The processed messages.
    /// * `Err(anyhow::Error)`: If an error occurred during processing.
    async fn process_messages(
        &self,
        messages: Vec<OpenAiChatMessage>,
    ) -> Result<Vec<OpenAiChatMessage>>;
}

pub trait MessageProcessor {
    fn process_message(
        &self,
        message: &OpenAiChatMessage,
        tool_call: Option<&ToolCall>,
    ) -> Result<Vec<OpenAiChatMessage>>;
}
