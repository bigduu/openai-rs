pub mod json_parser;
pub use json_parser::JsonParser;

use crate::openai_types::chat::OpenAiChatCompletionRequest;
use anyhow::Result;

/// Trait defining the contract for parsing incoming HTTP requests.
///
/// # Example
/// ```rust
/// use core::parser::RequestParser;
/// use core::openai_types::chat::{OpenAiChatCompletionRequest, OpenAiChatMessage};
/// use std::sync::Arc;
///
/// struct JsonParser;
///
/// impl RequestParser for JsonParser {
///     fn parse_request(&self, body: &[u8]) -> anyhow::Result<OpenAiChatCompletionRequest> {
///         let request: OpenAiChatCompletionRequest = serde_json::from_slice(body)?;
///         Ok(request)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let parser = JsonParser;
///     let body = r#"{
///         "model": "gpt-3.5-turbo",
///         "messages": [
///             {
///                 "role": "user",
///                 "content": "Hello"
///             }
///         ]
///     }"#.as_bytes();
///
///     let request = parser.parse_request(body)?;
///     assert_eq!(request.model, "gpt-3.5-turbo");
///     assert_eq!(request.messages[0].content, Some("Hello".to_string()));
///     assert_eq!(request.messages[0].role, "user".to_string());
///     Ok(())
/// }
/// ```
pub trait RequestParser: Send + Sync {
    /// Parses the given request body into an `OpenAiChatCompletionRequest`.
    ///
    /// # Arguments
    ///
    /// * `body`: The raw request body bytes.
    ///
    /// # Returns
    ///
    /// The parsed request or an error if parsing failed.
    fn parse_request(&self, body: &[u8]) -> Result<OpenAiChatCompletionRequest>;
}
