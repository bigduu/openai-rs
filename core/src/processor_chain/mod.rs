//! Module responsible for managing and executing the chain of processors.

use crate::openai_types::chat::OpenAiChatCompletionRequest;
use crate::processor::Processor;
use anyhow::Result;

pub struct ProcessorChain {
    processors: Vec<Box<dyn Processor>>,
}

impl ProcessorChain {
    pub fn new(processors: Vec<Box<dyn Processor>>) -> Self {
        ProcessorChain { processors }
    }

    pub async fn execute(
        &self,
        mut request: OpenAiChatCompletionRequest,
    ) -> Result<OpenAiChatCompletionRequest> {
        let mut messages = request.messages;

        for processor in &self.processors {
            messages = processor.process_messages(messages).await?;
        }

        request.messages = messages;
        Ok(request)
    }
}
