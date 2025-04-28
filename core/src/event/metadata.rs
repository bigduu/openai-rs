use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Additional metadata associated with an event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventMetadata {
    /// Timestamp of when the event was created
    pub timestamp: Option<String>,
    /// Source of the event (e.g., "openai", "claude", "custom")
    pub source: Option<String>,
    /// Additional custom metadata as key-value pairs
    pub custom: Option<Value>,
}
