use serde::{Deserialize, Serialize};

/// Represents the type of event in the stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Start of a message
    MessageStart,
    /// Content block within a message
    ContentBlock,
    /// End of a message
    MessageStop,
    /// Tool call event
    ToolCall,
    /// Tool result event
    ToolResult,
    /// Error event
    Error,
    /// Custom event type
    Custom(String),
}
