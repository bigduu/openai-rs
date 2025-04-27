use serde::{Deserialize, Serialize};

/// Configuration for stream handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// The maximum number of events to buffer
    pub buffer_size: usize,
    /// The timeout for stream operations in seconds
    pub timeout_secs: u64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            timeout_secs: 30,
        }
    }
}
