# Implementing Custom LLM Providers

This guide explains how to implement custom LLM providers for the LLM Proxy system.

## Overview

```text
┌──────────────┐     ┌───────────────┐     ┌────────────┐
│    Custom    │ ──> │  LLM Proxy    │ ──> │   Your     │
│   Provider   │     │    System     │     │ LLM Service│
└──────────────┘     └───────────────┘     └────────────┘
```

## Implementation Steps

1. Define Request/Response Types
2. Implement Core Traits
3. Add Configuration Support
4. Implement Error Handling
5. Add Tests

## Step 1: Define Types

### Request Type

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomRequest {
    // Required by LLMRequest trait
    messages: Vec<Message>,
    model: String,
    stream: bool,
    max_tokens: Option<u32>,
    
    // Provider-specific fields
    temperature: Option<f32>,
    top_p: Option<f32>,
    custom_field: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}
```

### Response Type

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomResponse {
    id: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    message: Message,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
```

## Step 2: Implement Core Traits

### LLMRequest Trait

```rust
use llm_proxy_core::LLMRequest;

impl LLMRequest for CustomRequest {
    fn messages(&self) -> Result<Value> {
        serde_json::to_value(&self.messages).map_err(Into::into)
    }

    fn model(&self) -> Result<String> {
        Ok(self.model.clone())
    }

    fn stream(&self) -> Result<bool> {
        Ok(self.stream)
    }

    fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    fn to_map(&self) -> Result<HashMap<String, Value>> {
        serde_json::to_value(self)
            .map_err(Into::into)
            .and_then(|v| match v {
                Value::Object(map) => Ok(map.into_iter()
                    .map(|(k, v)| (k, v))
                    .collect()),
                _ => Err(anyhow::anyhow!("Request is not an object").into()),
            })
    }

    fn to_value(&self) -> Result<Value> {
        serde_json::to_value(self).map_err(Into::into)
    }
}
```

### LLMClient Trait

```rust
use llm_proxy_core::{LLMClient, ResponseStream};

pub struct CustomProvider {
    client: reqwest::Client,
    token_provider: Box<dyn TokenProvider>,
    url_provider: Box<dyn UrlProvider>,
}

#[async_trait]
impl LLMClient<CustomRequest> for CustomProvider {
    async fn execute(&self, request: CustomRequest) -> Result<ResponseStream> {
        let url = self.url_provider.get_url().await?;
        let token = self.token_provider.get_token().await?;

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed: {} - {}",
                response.status(),
                response.text().await?
            ).into());
        }

        if request.stream() {
            self.handle_streaming_response(response).await
        } else {
            self.handle_normal_response(response).await
        }
    }
}

impl CustomProvider {
    async fn handle_streaming_response(
        &self,
        response: reqwest::Response,
    ) -> Result<ResponseStream> {
        let (tx, rx) = mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.into())).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn handle_normal_response(
        &self,
        response: reqwest::Response,
    ) -> Result<ResponseStream> {
        let (tx, rx) = mpsc::channel(1);
        let body = response.bytes().await?;
        tx.send(Ok(body)).await?;
        Ok(rx)
    }
}
```

## Step 3: Configuration Support

### Provider Configuration

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    base_url: String,
    token_env: String,
    timeout_secs: Option<u64>,
    max_retries: Option<u32>,
}

impl CustomProvider {
    pub fn from_config(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(
                config.timeout_secs.unwrap_or(300)
            ))
            .build()?;

        let token_provider = Box::new(
            EnvTokenProvider::new(&config.token_env)
        );
        let url_provider = Box::new(
            StaticUrlProvider::new(&config.base_url)
        );

        Ok(Self {
            client,
            token_provider,
            url_provider,
        })
    }
}
```

## Step 4: Error Handling

### Custom Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CustomProviderError {
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Request failed: {status} - {message}")]
    RequestError {
        status: u16,
        message: String,
    },

    #[error("Stream error: {0}")]
    StreamError(String),
}

impl From<CustomProviderError> for llm_proxy_core::Error {
    fn from(err: CustomProviderError) -> Self {
        match err {
            CustomProviderError::ParseError(e) => {
                Self::ParseError(e.to_string())
            }
            CustomProviderError::RequestError { status, message } => {
                Self::ProviderError(format!("{}: {}", status, message))
            }
            CustomProviderError::StreamError(e) => {
                Self::StreamError(e)
            }
        }
    }
}
```

### Error Handling in Provider

```rust
impl CustomProvider {
    async fn handle_error_response(
        &self,
        response: reqwest::Response,
    ) -> Result<()> {
        let status = response.status();
        let text = response.text().await?;

        Err(CustomProviderError::RequestError {
            status: status.as_u16(),
            message: text,
        }.into())
    }
}
```

## Step 5: Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn test_custom_provider() {
        // Setup mock server
        let mock = mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"1","choices":[...]}"#)
            .create();

        // Create provider
        let provider = CustomProvider::from_config(ProviderConfig {
            base_url: mockito::server_url(),
            token_env: "TEST_TOKEN".to_string(),
            timeout_secs: Some(30),
            max_retries: Some(3),
        }).unwrap();

        // Create request
        let request = CustomRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            model: "test-model".to_string(),
            stream: false,
            max_tokens: Some(100),
        };

        // Execute request
        let response_stream = provider.execute(request).await.unwrap();
        
        // Process response
        let mut response = String::new();
        while let Some(chunk) = response_stream.recv().await {
            response.push_str(
                &String::from_utf8_lossy(&chunk.unwrap())
            );
        }

        // Verify response
        assert!(response.contains("\"id\":\"1\""));
        mock.assert();
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_with_pipeline() {
        // Create pipeline components
        let parser = Arc::new(CustomRequestParser);
        let processors = vec![
            Arc::new(TokenLimitProcessor::new(2000)),
        ];
        let processor_chain = Arc::new(ProcessorChain::new(processors));
        let provider = Arc::new(CustomProvider::from_config(
            ProviderConfig::default()
        ).unwrap());

        // Create pipeline
        let pipeline = Pipeline::new(
            parser,
            processor_chain,
            provider,
        );

        // Test request
        let request = r#"{
            "messages": [{"role": "user", "content": "test"}],
            "model": "test-model"
        }"#;

        // Execute pipeline
        let response = pipeline.execute(
            Bytes::from(request)
        ).await.unwrap();

        // Verify response
        // ...
    }
}
```

## Best Practices

1. **Error Handling**
   - Use custom error types
   - Provide detailed error messages
   - Handle all error cases

2. **Configuration**
   - Make all parameters configurable
   - Use sensible defaults
   - Validate configuration

3. **Testing**
   - Write unit tests
   - Write integration tests
   - Use mock servers

4. **Documentation**
   - Document public API
   - Provide usage examples
   - Document configuration options

5. **Performance**
   - Use connection pooling
   - Implement proper timeouts
   - Handle streaming efficiently

## Common Pitfalls

1. **Token Management**
   - Don't hardcode tokens
   - Rotate tokens properly
   - Handle token expiration

2. **Error Handling**
   - Don't swallow errors
   - Provide context
   - Log appropriately

3. **Streaming**
   - Handle backpressure
   - Clean up resources
   - Handle disconnects

4. **Testing**
   - Don't rely on live API
   - Test error cases
   - Test configuration
