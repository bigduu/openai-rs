use crate::openai_types::chat::OpenAiChatMessage;
use anyhow::Result;
use async_trait::async_trait;

/// Defines the contract for a processor in the stream processing chain.
///
/// Processors are responsible for inspecting, modifying, filtering, or generating
/// messages as they flow through the pipeline.
/// They can perform tasks like content filtering, data enrichment (e.g., RAG), formatting,
/// or triggering side effects based on the message content.
///
/// Processors operate asynchronously, allowing them to perform I/O operations
/// (like calling external APIs or databases) without blocking the entire stream.
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
