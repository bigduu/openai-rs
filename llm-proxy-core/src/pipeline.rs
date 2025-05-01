use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    traits::{LLMClient, LLMRequest, ProcessorChain, RequestParser},
    types::{ResponseStream, Result},
};

/// Pipeline for handling LLM proxy requests.
///
/// The pipeline is the central component of the LLM proxy system. It coordinates
/// the flow of requests through three main stages:
///
/// 1. **Request Parsing**: Converting raw request bytes into structured requests
/// 2. **Request Processing**: Applying transformations and enhancements
/// 3. **LLM Execution**: Sending requests to the LLM service
///
/// Each stage is handled by trait objects, allowing for flexible configuration
/// and different implementations for different LLM services.
///
/// # Architecture
///
/// ```text
/// Raw Request → [RequestParser] → [ProcessorChain] → [LLMClient] → Response Stream
///                     ↓                  ↓                ↓
///              Structured Request → Modified Request → LLM Service
/// ```
///
/// # Example Usage
///
/// ```rust
/// # use std::sync::Arc;
/// # use anyhow::Result;
/// # use bytes::Bytes;
/// # use async_trait::async_trait;
/// # use llm_proxy_core::{Pipeline, RequestParser, ProcessorChain, LLMClient, ResponseStream, LLMRequest};
/// # use tokio::sync::mpsc;
/// #
/// # struct MyRequest;
/// # impl LLMRequest for MyRequest {
/// #     fn messages(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
/// #     fn model(&self) -> Result<String> { Ok("model".to_string()) }
/// #     fn stream(&self) -> Result<bool> { Ok(false) }
/// #     fn max_tokens(&self) -> Option<u32> { None }
/// #     fn to_map(&self) -> Result<std::collections::HashMap<String, serde_json::Value>> { Ok(std::collections::HashMap::new()) }
/// #     fn to_value(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
/// # }
/// #
/// # struct MyRequestParser;
/// # #[async_trait]
/// # impl RequestParser<MyRequest> for MyRequestParser {
/// #     async fn parse(&self, _body: Bytes) -> Result<MyRequest> {
/// #         Ok(MyRequest)
/// #     }
/// # }
/// #
/// # struct MyProcessor;
/// # #[async_trait]
/// # impl llm_proxy_core::Processor<MyRequest> for MyProcessor {
/// #     async fn process(&self, request: MyRequest) -> Result<MyRequest> {
/// #         Ok(request)
/// #     }
/// # }
/// #
/// # struct MyLLMClient;
/// # impl MyLLMClient {
/// #     fn new() -> Self { Self }
/// # }
/// # #[async_trait]
/// # impl LLMClient<MyRequest> for MyLLMClient {
/// #     async fn execute(&self, _request: MyRequest) -> Result<ResponseStream> {
/// #         let (tx, rx) = mpsc::channel(1);
/// #         Ok(rx)
/// #     }
/// # }
/// #
/// # async fn example() -> Result<()> {
/// # let pipeline = Pipeline::new(
/// #     Arc::new(MyRequestParser),
/// #     Arc::new(ProcessorChain::new(vec![Arc::new(MyProcessor)])),
/// #     Arc::new(MyLLMClient),
/// # );
/// // Create components
/// let parser = Arc::new(MyRequestParser);
/// let processors = vec![
///     Arc::new(MyProcessor),
/// ];
/// let processor_chain = Arc::new(ProcessorChain::new(processors));
/// let llm_client = Arc::new(MyLLMClient);
///
/// // Create pipeline
/// let pipeline = Pipeline::new(parser, processor_chain, llm_client);
///
/// // Execute request
/// let raw_request = Bytes::from("{}");
/// let response_stream = pipeline.execute(raw_request).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Implementing Custom Components
///
/// To create a custom pipeline implementation, you need to implement
/// the following traits:
///
/// 1. `RequestParser`: Parse raw requests into your request type
/// 2. `Processor`: Define request transformations (optional)
/// 3. `LLMClient`: Handle communication with your LLM service
///
/// See the documentation for each trait for implementation details.
#[derive(Clone)]
pub struct Pipeline<T: LLMRequest> {
    parser: Arc<dyn RequestParser<T>>,
    processor_chain: Arc<ProcessorChain<T>>,
    llm_client: Arc<dyn LLMClient<T>>,
    trace_id: Uuid,
}

impl<T: LLMRequest> Pipeline<T> {
    /// Create a new pipeline with the given components.
    ///
    /// # Arguments
    ///
    /// * `parser` - Component for parsing raw request bytes into structured requests
    /// * `processor_chain` - Chain of processors for transforming requests
    /// * `llm_client` - Client for communicating with the LLM service
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use anyhow::Result;
    /// # use bytes::Bytes;
    /// # use async_trait::async_trait;
    /// # use llm_proxy_core::{Pipeline, RequestParser, ProcessorChain, LLMClient, ResponseStream, LLMRequest};
    /// # use tokio::sync::mpsc;
    /// #
    /// # struct MyRequest;
    /// # impl LLMRequest for MyRequest {
    /// #     fn messages(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
    /// #     fn model(&self) -> Result<String> { Ok("model".to_string()) }
    /// #     fn stream(&self) -> Result<bool> { Ok(false) }
    /// #     fn max_tokens(&self) -> Option<u32> { None }
    /// #     fn to_map(&self) -> Result<std::collections::HashMap<String, serde_json::Value>> { Ok(std::collections::HashMap::new()) }
    /// #     fn to_value(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
    /// # }
    /// #
    /// # struct MyRequestParser;
    /// # #[async_trait]
    /// # impl RequestParser<MyRequest> for MyRequestParser {
    /// #     async fn parse(&self, _body: Bytes) -> Result<MyRequest> {
    /// #         Ok(MyRequest)
    /// #     }
    /// # }
    /// #
    /// # struct TokenLimitProcessor;
    /// # impl TokenLimitProcessor {
    /// #     fn new(_: u32) -> Self { Self }
    /// # }
    /// # #[async_trait]
    /// # impl llm_proxy_core::Processor<MyRequest> for TokenLimitProcessor {
    /// #     async fn process(&self, request: MyRequest) -> Result<MyRequest> {
    /// #         Ok(request)
    /// #     }
    /// # }
    /// #
    /// # struct MyLLMClient;
    /// # impl MyLLMClient {
    /// #     fn new() -> Self { Self }
    /// # }
    /// # #[async_trait]
    /// # impl LLMClient<MyRequest> for MyLLMClient {
    /// #     async fn execute(&self, _request: MyRequest) -> Result<ResponseStream> {
    /// #         let (tx, rx) = mpsc::channel(1);
    /// #         Ok(rx)
    /// #     }
    /// # }
    ///
    /// // Create pipeline components
    /// let parser = Arc::new(MyRequestParser);
    /// let processor_chain = Arc::new(ProcessorChain::new(vec![
    ///     Arc::new(TokenLimitProcessor::new(2000)),
    /// ]));
    /// let llm_client = Arc::new(MyLLMClient::new());
    ///
    /// // Create pipeline
    /// let pipeline = Pipeline::new(parser, processor_chain, llm_client);
    /// ```
    pub fn new(
        parser: Arc<dyn RequestParser<T>>,
        processor_chain: Arc<ProcessorChain<T>>,
        llm_client: Arc<dyn LLMClient<T>>,
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
    /// This method orchestrates the flow of a request through the pipeline:
    ///
    /// 1. The raw request bytes are parsed into a structured request
    /// 2. The request is processed through the processor chain
    /// 3. The processed request is sent to the LLM service
    ///
    /// The entire process is traced with unique IDs and comprehensive logging
    /// at each stage.
    ///
    /// # Arguments
    ///
    /// * `request_body` - Raw bytes of the incoming request
    ///
    /// # Returns
    ///
    /// A channel receiver (`ResponseStream`) that will receive response chunks
    /// from the LLM service. The response format depends on the specific
    /// LLM service implementation.
    ///
    /// # Error Handling
    ///
    /// This method will return an error if any stage fails:
    /// - Request parsing fails
    /// - A processor returns an error
    /// - The LLM client encounters an error
    ///
    /// All errors are logged with the trace ID for debugging.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use anyhow::Result;
    /// # use bytes::Bytes;
    /// # use async_trait::async_trait;
    /// # use llm_proxy_core::{Pipeline, RequestParser, ProcessorChain, LLMClient, ResponseStream, LLMRequest};
    /// # use tokio::sync::mpsc;
    /// #
    /// # struct MyRequest;
    /// # impl LLMRequest for MyRequest {
    /// #     fn messages(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
    /// #     fn model(&self) -> Result<String> { Ok("model".to_string()) }
    /// #     fn stream(&self) -> Result<bool> { Ok(false) }
    /// #     fn max_tokens(&self) -> Option<u32> { None }
    /// #     fn to_map(&self) -> Result<std::collections::HashMap<String, serde_json::Value>> { Ok(std::collections::HashMap::new()) }
    /// #     fn to_value(&self) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
    /// # }
    /// #
    /// # struct MyRequestParser;
    /// # #[async_trait]
    /// # impl RequestParser<MyRequest> for MyRequestParser {
    /// #     async fn parse(&self, _body: Bytes) -> Result<MyRequest> {
    /// #         Ok(MyRequest)
    /// #     }
    /// # }
    /// #
    /// # struct MyProcessor;
    /// # #[async_trait]
    /// # impl llm_proxy_core::Processor<MyRequest> for MyProcessor {
    /// #     async fn process(&self, request: MyRequest) -> Result<MyRequest> {
    /// #         Ok(request)
    /// #     }
    /// # }
    /// #
    /// # struct MyLLMClient;
    /// # #[async_trait]
    /// # impl LLMClient<MyRequest> for MyLLMClient {
    /// #     async fn execute(&self, _request: MyRequest) -> Result<ResponseStream> {
    /// #         let (tx, rx) = mpsc::channel(1);
    /// #         Ok(rx)
    /// #     }
    /// # }
    /// #
    /// # async fn example() -> Result<()> {
    /// # let pipeline = Pipeline::new(
    /// #     Arc::new(MyRequestParser),
    /// #     Arc::new(ProcessorChain::new(vec![Arc::new(MyProcessor)])),
    /// #     Arc::new(MyLLMClient),
    /// # );
    /// // Execute request
    /// let raw_request = Bytes::from(r#"{"messages": [...]}"#);
    /// let response_stream = pipeline.execute(raw_request).await?;
    ///
    /// // Process response chunks
    /// while let Some(chunk) = response_stream.recv().await {
    ///     match chunk {
    ///         Ok(data) => println!("Received: {:?}", data),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, request_body: bytes::Bytes) -> Result<ResponseStream> {
        info!(
            trace_id = %self.trace_id,
            request_size = request_body.len(),
            "Starting pipeline execution"
        );

        // 1. Parse Request
        let parsed_request = self.parser.parse(request_body).await?;
        debug!(
            trace_id = %self.trace_id,
            "Request parsed"
        );

        // 2. Process Request
        let processed_request = self.processor_chain.execute(parsed_request).await?;
        debug!(
            trace_id = %self.trace_id,
            "Request processed through chain"
        );

        // 3. Forward to LLM
        let response_stream = match self.llm_client.execute(processed_request).await {
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
    use std::collections::HashMap;

    use super::*;
    use crate::{LLMRequest, Processor};
    use async_trait::async_trait;
    use bytes::Bytes;
    use serde_json::Value;
    use tokio::sync::mpsc;

    struct MockRequestParser;

    struct MockRequest;
    impl LLMRequest for MockRequest {
        fn messages(&self) -> Result<Value> {
            Ok(serde_json::json!({"test": "request"}))
        }

        fn model(&self) -> Result<String> {
            Ok("test_model".to_string())
        }

        fn stream(&self) -> Result<bool> {
            Ok(true)
        }

        fn max_tokens(&self) -> Option<u32> {
            Some(1000)
        }

        fn to_map(&self) -> Result<HashMap<String, Value>> {
            Ok(serde_json::json!({"test": "request"})
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect())
        }

        fn to_value(&self) -> Result<Value> {
            Ok(serde_json::json!({"test": "request"}))
        }
    }

    #[async_trait]
    impl RequestParser<MockRequest> for MockRequestParser {
        async fn parse(&self, _body: Bytes) -> Result<MockRequest> {
            Ok(MockRequest)
        }
    }

    struct MockProcessor;

    #[async_trait]
    impl Processor<MockRequest> for MockProcessor {
        async fn process(&self, request: MockRequest) -> Result<MockRequest> {
            Ok(request)
        }
    }

    struct MockLLMClient;

    #[async_trait]
    impl LLMClient<MockRequest> for MockLLMClient {
        async fn execute(&self, _request: MockRequest) -> Result<ResponseStream> {
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

        let result = pipeline.execute(Bytes::from("test")).await;
        assert!(result.is_ok());

        let mut rx = result.unwrap();
        let response = rx.recv().await.unwrap().unwrap();
        assert_eq!(response, Bytes::from("test response"));
    }
}
