mod json_parser;
pub use json_parser::JsonParser;

use crate::openai_types::chat::OpenAiChatCompletionRequest;
use anyhow::Result;

/// Defines the contract for parsing incoming HTTP requests into OpenAiChatCompletionRequest.
///
/// Implementations of this trait are responsible for converting raw request data
/// into a structured format that can be processed by the core forwarding logic.
///
/// This abstraction allows the core forwarding logic to remain decoupled from
/// specific parsing mechanisms. Different strategies (JSON, XML, custom formats)
/// can be implemented and swapped easily.
pub trait RequestParser: Send + Sync {
    /// Parses the raw request body into a OpenAiChatCompletionRequest.
    ///
    /// # Arguments
    ///
    /// * `request_body` - The raw bytes of the HTTP request body
    ///
    /// # Returns
    ///
    /// * `Ok(OpenAiChatCompletionRequest)`: Containing the parsed request if successful
    /// * `Err(anyhow::Error)`: If an error occurred during parsing
    fn parse(&self, request_body: &[u8]) -> Result<OpenAiChatCompletionRequest>;
}
