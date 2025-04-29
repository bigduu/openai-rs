use crate::openai_types::chat::OpenAiChatCompletionRequest;
use crate::parser::RequestParser;
use anyhow::Result;
use serde_json::from_slice;

/// A JSON parser implementation for OpenAI chat completion requests.
///
/// This parser uses serde_json to deserialize the request body into
/// an OpenAiChatCompletionRequest struct.
///
/// # Example
/// ```
/// use core::parser::json_parser::JsonParser;
/// use core::parser::RequestParser;
///
/// let parser = JsonParser::new();
/// let body = r#"{"model": "gpt-3.5-turbo", "messages": []}"#.as_bytes();
/// let request = parser.parse_request(body).unwrap();
/// assert_eq!(request.model, "gpt-3.5-turbo");
/// ```
pub struct JsonParser;

impl RequestParser for JsonParser {
    fn parse_request(&self, body: &[u8]) -> Result<OpenAiChatCompletionRequest> {
        // Directly deserialize the request body into OpenAiChatCompletionRequest
        from_slice(body).map_err(Into::into)
    }
}

impl JsonParser {
    /// Creates a new JsonParser instance.
    pub fn new() -> Self {
        JsonParser
    }
}
