use crate::{
    client_provider::ClientProvider,
    forwarder::StreamForwarder,
    parser::RequestParser,
    processor_chain::ProcessorChain,
    sse_provider::SseProvider,
    token_provider::TokenProvider,
    url_provider::{StaticUrlProvider, UrlProvider},
};
use anyhow::Result;
use bytes::Bytes;
use futures::Stream;
use std::{pin::Pin, sync::Arc};

/// A context that holds all necessary components for processing streaming proxy requests.
/// This is the main entry point for handling OpenAI API requests with streaming support.
///
/// # Example
/// ```rust
/// use core::context::{StreamingProxyContext, StreamingProxyContextBuilder};
/// use core::token_provider::StaticTokenProvider;
/// use std::sync::Arc;
///
/// // Create a default context with default providers
/// let context = StreamingProxyContext::default();
///
/// // Or use the builder pattern for custom configuration
/// let context = StreamingProxyContextBuilder::new()
///     .with_token_provider(Arc::new(StaticTokenProvider::new("your-api-key".to_string())))
///     .build();
/// ```
pub struct StreamingProxyContext {
    pub client_provider: Arc<dyn ClientProvider>,
    pub url_provider: Arc<dyn UrlProvider>,
    pub token_provider: Arc<dyn TokenProvider>,
    pub sse_provider: Arc<dyn SseProvider>,
    pub forwarder: Arc<StreamForwarder>,
    pub parser: Arc<dyn RequestParser>,
    pub processor_chain: Arc<ProcessorChain>,
}

impl StreamingProxyContext {
    /// Process a request and return a stream of bytes.
    /// This is the main method to handle incoming requests and forward them to OpenAI.
    ///
    /// # Arguments
    /// * `req_body` - The raw request body bytes containing the chat completion request
    ///
    /// # Returns
    /// A Result containing a pinned boxed stream of bytes, or an error if processing fails
    ///
    /// # Example
    /// ```rust
    /// use core::context::StreamingProxyContext;
    /// use bytes::Bytes;
    /// use futures::StreamExt;
    ///
    /// async fn handle_request(context: &StreamingProxyContext, request_body: Bytes) {
    ///     let mut stream = context.process_request(request_body).await.unwrap();
    ///     while let Some(chunk) = stream.next().await {
    ///         // Process each chunk of the response
    ///     }
    /// }
    /// ```
    pub async fn process_request(
        &self,
        req_body: Bytes,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        // 1. Parse Request
        let req_body = req_body.to_vec();
        let openai_chat_completion_request = match self.parser.parse_request(&req_body) {
            Ok(request) => request,
            Err(e) => return Err(e),
        };

        // 2. Processor Chain
        let processed_messages = match self
            .processor_chain
            .execute(openai_chat_completion_request)
            .await
        {
            Ok(messages) => messages,
            Err(e) => return Err(anyhow::anyhow!("Error during processing: {}", e)),
        };

        // 3. & 4. Forward to OpenAI and get response stream
        let response_stream = match self
            .forwarder
            .forward(
                processed_messages,
                &*self.token_provider,
                &*self.url_provider,
            )
            .await
        {
            Ok(stream) => stream,
            Err(e) => return Err(anyhow::anyhow!("Error forwarding request: {}", e)),
        };

        // 5. Convert to HTTP stream
        self.sse_provider
            .to_http_stream(Box::pin(response_stream))
            .await
    }
}

/// Builder for StreamingProxyContext that allows for flexible configuration of all components.
/// Use this to create a customized StreamingProxyContext with specific providers and processors.
///
/// # Example
/// ```rust
/// use core::context::StreamingProxyContextBuilder;
/// use core::token_provider::StaticTokenProvider;
/// use core::url_provider::StaticUrlProvider;
/// use std::sync::Arc;
///
/// let context = StreamingProxyContextBuilder::new()
///     .with_token_provider(Arc::new(StaticTokenProvider::new("your-api-key".to_string())))
///     .with_url_provider(Arc::new(StaticUrlProvider::new("https://api.openai.com/v1/chat/completions".to_string())))
///     .build();
/// ```
pub struct StreamingProxyContextBuilder {
    client_provider: Option<Arc<dyn ClientProvider>>,
    url_provider: Option<Arc<dyn UrlProvider>>,
    token_provider: Option<Arc<dyn TokenProvider>>,
    sse_provider: Option<Arc<dyn SseProvider>>,
    parser: Option<Arc<dyn RequestParser>>,
    processor_chain: Option<Arc<ProcessorChain>>,
}

impl StreamingProxyContextBuilder {
    /// Creates a new builder with all components unset.
    /// Use the with_* methods to configure specific components.
    pub fn new() -> Self {
        Self {
            client_provider: None,
            url_provider: None,
            token_provider: None,
            sse_provider: None,
            parser: None,
            processor_chain: None,
        }
    }

    /// Sets a custom client provider for making HTTP requests.
    /// If not set, a default StaticClientProvider will be used.
    pub fn with_client_provider(mut self, provider: Arc<dyn ClientProvider>) -> Self {
        self.client_provider = Some(provider);
        self
    }

    /// Sets a custom URL provider for determining the OpenAI API endpoint.
    /// If not set, a default StaticUrlProvider pointing to the OpenAI chat completions endpoint will be used.
    pub fn with_url_provider(mut self, provider: Arc<dyn UrlProvider>) -> Self {
        self.url_provider = Some(provider);
        self
    }

    /// Sets a custom token provider for OpenAI API authentication.
    /// If not set, a default StaticTokenProvider with a placeholder key will be used.
    pub fn with_token_provider(mut self, provider: Arc<dyn TokenProvider>) -> Self {
        self.token_provider = Some(provider);
        self
    }

    /// Sets a custom SSE provider for handling Server-Sent Events.
    /// If not set, a default DefaultSseProvider will be used.
    pub fn with_sse_provider(mut self, provider: Arc<dyn SseProvider>) -> Self {
        self.sse_provider = Some(provider);
        self
    }

    /// Sets a custom request parser for parsing incoming requests.
    /// If not set, a default JsonParser will be used.
    pub fn with_parser(mut self, parser: Arc<dyn RequestParser>) -> Self {
        self.parser = Some(parser);
        self
    }

    /// Sets a custom processor chain for processing messages before forwarding.
    /// If not set, an empty processor chain will be used.
    pub fn with_processor_chain(mut self, chain: Arc<ProcessorChain>) -> Self {
        self.processor_chain = Some(chain);
        self
    }

    /// Builds the StreamingProxyContext with the configured components.
    /// Any unset components will use their default implementations.
    pub fn build(self) -> StreamingProxyContext {
        let client_provider = self
            .client_provider
            .unwrap_or_else(|| Arc::new(crate::client_provider::StaticClientProvider::new()));

        let url_provider = self.url_provider.unwrap_or_else(|| {
            Arc::new(StaticUrlProvider::new(
                "https://api.openai.com/v1/chat/completions".to_string(),
            )) as Arc<dyn UrlProvider>
        });

        let token_provider = self.token_provider.unwrap_or_else(|| {
            Arc::new(crate::token_provider::StaticTokenProvider::new(
                "YOUR_STATIC_OPENAI_KEY".to_string(),
            ))
        });

        let sse_provider = self.sse_provider.unwrap_or_else(|| {
            Arc::new(crate::sse_provider::default_sse::DefaultSseProvider::new())
        });

        let parser = self
            .parser
            .unwrap_or_else(|| Arc::new(crate::parser::JsonParser::new()));

        let processor_chain = self
            .processor_chain
            .unwrap_or_else(|| Arc::new(ProcessorChain::new(vec![])));

        let forwarder = Arc::new(StreamForwarder::new(client_provider.clone()));

        StreamingProxyContext {
            client_provider,
            url_provider,
            token_provider,
            sse_provider,
            forwarder,
            parser,
            processor_chain,
        }
    }
}

impl Default for StreamingProxyContext {
    fn default() -> Self {
        StreamingProxyContextBuilder::new().build()
    }
}

impl Clone for StreamingProxyContext {
    fn clone(&self) -> Self {
        Self {
            client_provider: self.client_provider.clone(),
            url_provider: self.url_provider.clone(),
            token_provider: self.token_provider.clone(),
            sse_provider: self.sse_provider.clone(),
            forwarder: self.forwarder.clone(),
            parser: self.parser.clone(),
            processor_chain: self.processor_chain.clone(),
        }
    }
}
