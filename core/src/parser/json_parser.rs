use super::RequestParser;
use crate::openai_types::chat::OpenAiChatCompletionRequest;
use anyhow::Result;
use serde_json::from_slice;

/// Default JSON implementation of the RequestParser trait.
///
/// This parser expects the request body to be a JSON-serialized OpenAiChatCompletionRequest.
pub struct JsonParser;

impl JsonParser {
    /// Creates a new instance of JsonParser.
    pub fn new() -> Self {
        JsonParser
    }
}

impl RequestParser for JsonParser {
    fn parse(&self, request_body: &[u8]) -> Result<OpenAiChatCompletionRequest> {
        // Directly deserialize the request body into OpenAiChatCompletionRequest
        from_slice(request_body).map_err(Into::into)
    }
}
