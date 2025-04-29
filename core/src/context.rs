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

/// A context that holds all necessary components for processing streaming proxy requests
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
    /// Process a request and return a stream of bytes
    pub async fn process_request(
        &self,
        req_body: Bytes,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        // 1. Parse Request
        let openai_chat_completion_request = match self.parser.parse(&req_body) {
            Ok(request) => request,
            Err(e) => return Err(anyhow::anyhow!("Invalid request format: {}", e)),
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

/// Builder for StreamingProxyContext
pub struct StreamingProxyContextBuilder {
    client_provider: Option<Arc<dyn ClientProvider>>,
    url_provider: Option<Arc<dyn UrlProvider>>,
    token_provider: Option<Arc<dyn TokenProvider>>,
    sse_provider: Option<Arc<dyn SseProvider>>,
    parser: Option<Arc<dyn RequestParser>>,
    processor_chain: Option<Arc<ProcessorChain>>,
}

impl StreamingProxyContextBuilder {
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

    pub fn with_client_provider(mut self, provider: Arc<dyn ClientProvider>) -> Self {
        self.client_provider = Some(provider);
        self
    }

    pub fn with_url_provider(mut self, provider: Arc<dyn UrlProvider>) -> Self {
        self.url_provider = Some(provider);
        self
    }

    pub fn with_token_provider(mut self, provider: Arc<dyn TokenProvider>) -> Self {
        self.token_provider = Some(provider);
        self
    }

    pub fn with_sse_provider(mut self, provider: Arc<dyn SseProvider>) -> Self {
        self.sse_provider = Some(provider);
        self
    }

    pub fn with_parser(mut self, parser: Arc<dyn RequestParser>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_processor_chain(mut self, chain: Arc<ProcessorChain>) -> Self {
        self.processor_chain = Some(chain);
        self
    }

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
