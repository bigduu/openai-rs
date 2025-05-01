use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    traits::{LLMClient, ProcessorChain, RequestParser},
    types::{ResponseStream, Result},
};

/// Pipeline for handling LLM proxy requests.
///
/// The pipeline coordinates the different stages of request handling:
/// 1. Request parsing
/// 2. Request processing through the processor chain
/// 3. Forwarding to the LLM service
///
/// All components are trait objects, allowing for flexible configuration
/// and different implementations for different LLM services.
#[derive(Clone)]
pub struct Pipeline {
    parser: Arc<dyn RequestParser>,
    processor_chain: Arc<ProcessorChain>,
    llm_client: Arc<dyn LLMClient>,
    trace_id: Uuid,
}

impl Pipeline {
    /// Create a new pipeline with the given components.
    ///
    /// # Arguments
    /// * `parser` - Component for parsing raw request bytes
    /// * `processor_chain` - Chain of processors for modifying requests
    /// * `llm_client` - Client for communicating with the LLM service
    pub fn new(
        parser: Arc<dyn RequestParser>,
        processor_chain: Arc<ProcessorChain>,
        llm_client: Arc<dyn LLMClient>,
    ) -> Self {
        Self {
            parser,
            processor_chain,
            llm_client,
            trace_id: Uuid::new_v4(),
        }
    }

    /// Execute the pipeline on a request.
    ///
    /// # Arguments
    /// * `request_body` - Raw bytes of the request
    /// * `stream` - Whether to request a streaming response (determined by server based on route config)
    ///
    /// # Returns
    /// A channel receiver that will receive the raw response bytes from the LLM service
    pub async fn execute(
        &self,
        request_body: bytes::Bytes,
        stream: bool,
    ) -> Result<ResponseStream> {
        info!(
            trace_id = %self.trace_id,
            request_size = request_body.len(),
            stream = stream,
            "Starting pipeline execution"
        );

        // 1. Parse Request
        let (parsed_request, stream_requested) = self.parser.parse(request_body).await?;
        debug!(
            trace_id = %self.trace_id,
            stream_requested = stream_requested,
            "Request parsed"
        );

        // Warn if streaming was requested but not allowed by route config
        if stream_requested && !stream {
            warn!(
                trace_id = %self.trace_id,
                "Streaming was requested but is not allowed by route configuration"
            );
        }

        // 2. Process Request
        let processed_request = self.processor_chain.execute(parsed_request).await?;
        debug!(
            trace_id = %self.trace_id,
            "Request processed through chain"
        );

        // 3. Forward to LLM
        let response_stream = match self.llm_client.execute(processed_request, stream).await {
            Ok(stream) => stream,
            Err(e) => {
                error!(
                    trace_id = %self.trace_id,
                    error = %e,
                    "Error executing request with LLM client"
                );
                return Err(e);
            }
        };

        info!(
            trace_id = %self.trace_id,
            "Pipeline execution completed successfully"
        );
        Ok(response_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use async_trait::async_trait;
    use bytes::Bytes;
    use serde_json::Value;
    use tokio::sync::mpsc;

    struct MockRequestParser;

    #[async_trait]
    impl RequestParser for MockRequestParser {
        async fn parse(&self, _body: Bytes) -> Result<(Value, bool)> {
            Ok((serde_json::json!({"test": "request"}), true))
        }
    }

    struct MockProcessor;

    #[async_trait]
    impl crate::traits::Processor for MockProcessor {
        async fn process(&self, request: Value) -> Result<Value> {
            Ok(request)
        }
    }

    struct MockLLMClient;

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn execute(&self, _request: Value, _stream: bool) -> Result<ResponseStream> {
            let (tx, rx) = mpsc::channel(1);
            let _ = tx.send(Ok(Bytes::from("test response"))).await;
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let pipeline = Pipeline::new(
            Arc::new(MockRequestParser),
            Arc::new(ProcessorChain::new(vec![Arc::new(MockProcessor)])),
            Arc::new(MockLLMClient),
        );

        let result = pipeline.execute(Bytes::from("test"), true).await;
        assert!(result.is_ok());

        let mut rx = result.unwrap();
        let response = rx.recv().await.unwrap().unwrap();
        assert_eq!(response, Bytes::from("test response"));
    }
}
