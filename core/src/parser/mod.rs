mod json_parser;
pub use json_parser::JsonParser;

use crate::event::InternalStreamEvent;
use anyhow::Result;

/// Defines the contract for parsing incoming HTTP requests into InternalStreamEvents.
///
/// Implementations of this trait are responsible for converting raw request data
/// into a structured format that can be processed by the core forwarding logic.
///
/// This abstraction allows the core forwarding logic to remain decoupled from
/// specific parsing mechanisms. Different strategies (JSON, XML, custom formats)
/// can be implemented and swapped easily.
pub trait RequestParser: Send + Sync {
    /// Parses the raw request body into a vector of InternalStreamEvents.
    ///
    /// # Arguments
    ///
    /// * `request_body` - The raw bytes of the HTTP request body
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<InternalStreamEvent>)`: Containing the parsed events if successful
    /// * `Err(anyhow::Error)`: If an error occurred during parsing
    fn parse(&self, request_body: &[u8]) -> Result<Vec<InternalStreamEvent>>;
}
