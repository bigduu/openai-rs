# Implementing Custom Providers

This guide walks through the process of implementing custom providers for the LLM Proxy system. It includes step-by-step instructions, code examples, and best practices.

## Table of Contents

1. [Provider Interface](#provider-interface)
2. [Basic Implementation](#basic-implementation)
3. [Streaming Support](#streaming-support)
4. [Error Handling](#error-handling)
5. [Configuration](#configuration)
6. [Testing](#testing)
7. [Best Practices](#best-practices)

## Provider Interface

The core of the provider system is the `LLMClient` trait:

```rust
#[async_trait::async_trait]
pub trait LLMClient<Request> {
    async fn execute(&self, request: Request) -> Result<ResponseStream>;
    
    // Optional methods with default implementations
    async fn validate(&self) -> Result<()> {
        Ok(())
    }
    
    fn supports_streaming(&self) -> bool;
}
```

## Basic Implementation

Here's a step-by-step guide to implementing a basic provider:

1. **Define Request/Response Types**

    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CustomRequest {
        pub prompt: String,
        pub max_tokens: Option<u32>,
        pub temperature: Option<f32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CustomResponse {
        pub text: String,
        pub finish_reason: Option<String>,
    }
    ```

2. **Create Provider Struct**

    ```rust
    pub struct CustomProvider {
        client: reqwest::Client,
        base_url: String,
        api_key: String,
    }

    impl CustomProvider {
        pub fn new(base_url: String, api_key: String) -> Self {
            Self {
                client: reqwest::Client::new(),
                base_url,
                api_key,
            }
        }
    }
    ```

3. **Implement LLMClient Trait**

```rust
#[async_trait::async_trait]
impl LLMClient<CustomRequest> for CustomProvider {
    async fn execute(&self, request: CustomRequest) -> Result<ResponseStream> {
        let response = self.client
            .post(&format!("{}/v1/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(Error::ProviderError(format!(
                "Request failed: {}", 
                response.status()
            )));
        }
        
        let body: CustomResponse = response.json().await?;
        Ok(ResponseStream::from_response(body))
    }
    
    fn supports_streaming(&self) -> bool {
        false // Set to true if implementing streaming
    }
}
```

## Streaming Support

To add streaming support:

1. **Implement Streaming Response Handler**

    ```rust
    impl CustomProvider {
        async fn handle_stream(
            response: reqwest::Response
        ) -> Result<ResponseStream> {
            let stream = response
                .bytes_stream()
                .map_err(|e| Error::StreamError(e.to_string()))
                .map(|chunk| {
                    chunk.map_err(|e| Error::StreamError(e.to_string()))
                })
                .map(|bytes| {
                    // Parse SSE data
                    let data = String::from_utf8_lossy(&bytes?);
                    CustomResponse::from_sse(&data)
                });
                
            Ok(ResponseStream::new(stream))
        }
    }
    ```

2. **Update Execute Method**

```rust
#[async_trait::async_trait]
impl LLMClient<CustomRequest> for CustomProvider {
    async fn execute(&self, request: CustomRequest) -> Result<ResponseStream> {
        let response = self.client
            .post(&format!("{}/v1/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(Error::ProviderError(format!(
                "Request failed: {}", 
                response.status()
            )));
        }
        
        if request.stream {
            Self::handle_stream(response).await
        } else {
            let body: CustomResponse = response.json().await?;
            Ok(ResponseStream::from_response(body))
        }
    }
    
    fn supports_streaming(&self) -> bool {
        true
    }
}
```

## Error Handling

Implement proper error handling:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CustomProviderError {
    #[error("API request failed: {0}")]
    RequestFailed(String),
    
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    
    #[error("Stream error: {0}")]
    StreamError(String),
}

impl From<CustomProviderError> for Error {
    fn from(err: CustomProviderError) -> Self {
        Error::ProviderError(err.to_string())
    }
}
```

## Configuration

1. **Define Configuration**

    ```rust
    #[derive(Debug, Deserialize)]
    pub struct CustomProviderConfig {
        pub base_url: String,
        pub api_key_env: String,
        pub timeout: Option<Duration>,
    }
    ```

2. **Implement From Configuration**

```rust
impl CustomProvider {
    pub fn from_config(config: CustomProviderConfig) -> Result<Self> {
        let api_key = std::env::var(&config.api_key_env)
            .map_err(|_| Error::ConfigError(format!(
                "Missing API key in environment: {}", 
                config.api_key_env
            )))?;
            
        let client = reqwest::Client::builder()
            .timeout(config.timeout.unwrap_or(Duration::from_secs(30)))
            .build()?;
            
        Ok(Self {
            client,
            base_url: config.base_url,
            api_key,
        })
    }
}
```

## Testing

1. **Unit Tests**

    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        use mockito::mock;
        
        #[tokio::test]
        async fn test_basic_completion() {
            let mock = mock("POST", "/v1/completions")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{"text": "test response"}"#)
                .create();
                
            let provider = CustomProvider::new(
                mockito::server_url(),
                "test-key".to_string()
            );
            
            let request = CustomRequest {
                prompt: "test".to_string(),
                max_tokens: Some(10),
                temperature: Some(0.7),
            };
            
            let response = provider.execute(request).await.unwrap();
            assert!(response.is_success());
            
            mock.assert();
        }
    }
    ```

2. **Integration Tests**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_streaming_response() {
        let provider = CustomProvider::from_config(
            CustomProviderConfig {
                base_url: "https://api.example.com".to_string(),
                api_key_env: "TEST_API_KEY".to_string(),
                timeout: None,
            }
        ).unwrap();
        
        let request = CustomRequest {
            prompt: "test streaming".to_string(),
            stream: true,
            max_tokens: Some(10),
        };
        
        let mut stream = provider.execute(request).await.unwrap();
        
        while let Some(chunk) = stream.next().await {
            assert!(chunk.is_ok());
        }
    }
}
```

## Best Practices

1. **Error Handling**
   - Use custom error types
   - Provide detailed error messages
   - Handle all possible failure cases

2. **Configuration**
   - Use environment variables for secrets
   - Make timeouts configurable
   - Validate configuration at startup

3. **Performance**
   - Reuse HTTP clients
   - Implement proper timeouts
   - Handle backpressure in streams

4. **Testing**
   - Write comprehensive unit tests
   - Include integration tests
   - Test error cases
   - Mock external services

5. **Documentation**
   - Document public interfaces
   - Include usage examples
   - Document configuration options
   - Add inline comments for complex logic

6. **Security**
   - Never log API keys
   - Validate input
   - Sanitize output
   - Use HTTPS for API calls

## Example Provider Implementation

Here's a complete example of a custom provider:

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct CustomRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomResponse {
    text: String,
    finish_reason: Option<String>,
}

pub struct CustomProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

#[async_trait]
impl LLMClient<CustomRequest> for CustomProvider {
    async fn execute(&self, request: CustomRequest) -> Result<ResponseStream> {
        let response = self.client
            .post(&format!("{}/v1/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", if request.stream {
                "text/event-stream"
            } else {
                "application/json"
            })
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(Error::ProviderError(format!(
                "Request failed: {}", 
                response.status()
            )));
        }
        
        if request.stream {
            Self::handle_stream(response).await
        } else {
            let body: CustomResponse = response.json().await?;
            Ok(ResponseStream::from_response(body))
        }
    }
    
    fn supports_streaming(&self) -> bool {
        true
    }
    
    async fn validate(&self) -> Result<()> {
        // Perform validation like checking API key
        let response = self.client
            .get(&format!("{}/v1/validate", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(Error::ProviderError(
                "Failed to validate API key".to_string()
            ));
        }
        
        Ok(())
    }
}

impl CustomProvider {
    pub fn new(base_url: String, api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url,
            api_key,
        }
    }
    
    async fn handle_stream(
        response: reqwest::Response
    ) -> Result<ResponseStream> {
        let stream = response
            .bytes_stream()
            .map_err(|e| Error::StreamError(e.to_string()))
            .map(|chunk| {
                chunk.map_err(|e| Error::StreamError(e.to_string()))
            })
            .map(|bytes| {
                let data = String::from_utf8_lossy(&bytes?);
                CustomResponse::from_sse(&data)
            });
            
        Ok(ResponseStream::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    
    #[tokio::test]
    async fn test_basic_request() {
        let mock = mock("POST", "/v1/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"text": "test response"}"#)
            .create();
            
        let provider = CustomProvider::new(
            mockito::server_url(),
            "test-key".to_string()
        );
        
        let request = CustomRequest {
            prompt: "test".to_string(),
            max_tokens: Some(10),
            temperature: Some(0.7),
            stream: false,
        };
        
        let response = provider.execute(request).await.unwrap();
        assert!(response.is_success());
        
        mock.assert();
    }
}
```

This implementation guide should help you create robust and maintainable custom providers for the LLM Proxy system. Remember to follow the best practices and thoroughly test your implementation.
