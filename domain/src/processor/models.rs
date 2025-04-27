use crate::event::InternalStreamEvent;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::VecDeque;

/// Defines the contract for a processor in the stream processing chain.
///
/// Processors are responsible for inspecting, modifying, filtering, or generating
/// `InternalStreamEvent`s as they flow through the pipeline. They can perform
/// tasks like content filtering, data enrichment (e.g., RAG), formatting,
/// or triggering side effects based on the event content.
///
/// Processors operate asynchronously, allowing them to perform I/O operations
/// (like calling external APIs or databases) without blocking the entire stream.
#[async_trait]
pub trait Processor: Send + Sync {
    /// Processes a single incoming event.
    ///
    /// # Arguments
    ///
    /// * `event`: A mutable reference to the `InternalStreamEvent` currently being processed.
    ///   The processor can modify this event in place.
    /// * `output_queue`: A mutable reference to a `VecDeque` where the processor can
    ///   push zero or more new `InternalStreamEvent`s. These events will be processed
    ///   by subsequent stages in the pipeline *after* the current event (if it's not dropped).
    async fn process(
        &self,
        event: &mut InternalStreamEvent,
        output_queue: &mut VecDeque<InternalStreamEvent>,
    ) -> Result<()>;
}

/// Represents the configuration for a processor
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub name: String,
    pub enabled: bool,
    // Add other configuration fields as needed
}
