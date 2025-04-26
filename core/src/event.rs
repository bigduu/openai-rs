use serde::{Deserialize, Serialize};

/// Represents a standardized event structure used internally within the processing pipeline.
/// This allows different components to work with a consistent data format, regardless of
/// the original source (e.g., OpenAI API, Claude API) or the target format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalStreamEvent {
    /// The role associated with the event content (e.g., "user", "assistant", "system", "tool").
    /// Optional because some events might not have a specific role (e.g., control signals).
    pub role: Option<String>,

    /// The textual content of the event.
    /// Optional as some events might represent actions or metadata without direct text content.
    pub content: Option<String>,
    // Potentially add other fields later, like:
    // pub event_type: String, // e.g., "message_start", "content_block", "message_stop", "tool_call"
    // pub metadata: Option<serde_json::Value>, // For additional context
}

impl InternalStreamEvent {
    /// Creates a new event with the given role and content.
    pub fn new(role: Option<String>, content: Option<String>) -> Self {
        InternalStreamEvent { role, content }
    }

    /// Creates a simple user message event.
    pub fn new_user(content: String) -> Self {
        Self::new(Some("user".to_string()), Some(content))
    }

    /// Creates a simple assistant message event.
    pub fn new_assistant(content: String) -> Self {
        Self::new(Some("assistant".to_string()), Some(content))
    }
}
