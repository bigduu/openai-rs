use anyhow::Result;
use domain::{event::InternalStreamEvent, processor::Processor, stream::StreamConfig};
use std::collections::VecDeque;

/// Manages a single conversation stream and its associated processor chain
pub struct ConversationStream {
    /// The chain of processors that will handle events
    processors: Vec<Box<dyn Processor>>,
    /// Configuration for the stream
    config: StreamConfig,
}

impl ConversationStream {
    /// Creates a new conversation stream with the given processors
    pub fn new(processors: Vec<Box<dyn Processor>>) -> Self {
        Self {
            processors,
            config: StreamConfig::default(),
        }
    }

    /// Creates a new conversation stream with custom configuration
    pub fn with_config(processors: Vec<Box<dyn Processor>>, config: StreamConfig) -> Self {
        Self { processors, config }
    }

    /// Processes an event through the processor chain
    pub async fn handle(
        &self,
        event: InternalStreamEvent,
    ) -> Result<VecDeque<InternalStreamEvent>> {
        let mut queue = VecDeque::with_capacity(self.config.buffer_size);
        queue.push_back(event);

        for processor in &self.processors {
            if let Some(mut evt) = queue.pop_front() {
                processor.process(&mut evt, &mut queue).await?;
                queue.push_back(evt);
            }
        }

        Ok(queue)
    }
}
