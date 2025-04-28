use serde::{Deserialize, Serialize};

mod event_type;
mod metadata;

pub use event_type::EventType;
pub use metadata::EventMetadata;

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

    /// The type of the event
    pub event_type: Option<EventType>,

    /// Additional metadata associated with the event
    pub metadata: Option<EventMetadata>,
}

impl InternalStreamEvent {
    /// Creates a new event with the given role and content.
    pub fn new(role: Option<String>, content: Option<String>) -> Self {
        InternalStreamEvent {
            role,
            content,
            event_type: None,
            metadata: None,
        }
    }

    /// Creates a simple user message event.
    pub fn new_user(content: String) -> Self {
        Self::new(Some("user".to_string()), Some(content))
    }

    /// Creates a simple assistant message event.
    pub fn new_assistant(content: String) -> Self {
        Self::new(Some("assistant".to_string()), Some(content))
    }

    /// Creates a simple system message event.
    pub fn new_system(content: String) -> Self {
        Self::new(Some("system".to_string()), Some(content))
    }

    /// Sets the event type
    pub fn with_event_type(mut self, event_type: EventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Sets the metadata
    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
