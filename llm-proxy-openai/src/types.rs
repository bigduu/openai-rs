use llm_proxy_core::{LLMRequest, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A request to the `OpenAI` chat completions API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatCompletionRequest {
    /// The messages to generate completions for
    pub messages: Vec<Message>,
    /// The model to use (e.g., "gpt-4", "gpt-3.5-turbo")
    pub model: String,
    /// Whether to stream responses token by token
    #[serde(default)]
    pub stream: bool,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Temperature for response randomness
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Functions that the model may call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,
    /// Additional model parameters
    #[serde(flatten)]
    pub additional_params: HashMap<String, serde_json::Value>,
}

/// A message in a chat completion request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    /// The role of the message sender (system, user, assistant, or function)
    pub role: String,
    /// The content of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Name of the function that was called
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Function call in the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

/// A function that can be called by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    pub description: String,
    /// Parameters the function accepts, in JSON Schema format
    pub parameters: serde_json::Value,
}

/// A function call in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function to call
    pub name: String,
    /// Arguments to pass to the function, as a JSON string
    pub arguments: String,
}

/// A chunk in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// The ID of this chunk
    pub id: String,
    /// The object type (always "chat.completion.chunk")
    pub object: String,
    /// Unix timestamp of when the chunk was created
    pub created: u64,
    /// The model that generated this chunk
    pub model: String,
    /// Array of choices (usually just one) in this chunk
    pub choices: Vec<StreamChoice>,
}

/// A choice in a streaming response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    /// Index of this choice
    pub index: usize,
    /// The delta (changes) in this chunk
    pub delta: StreamDelta,
    /// Reason why this chunk ended, if it did
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// The changes in a streaming response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    /// Role of the message (usually only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Content of the message (the actual token)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Function call, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

/// Error response from the `OpenAI` API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The error details
    pub error: ErrorDetails,
}

/// Details of an error from the `OpenAI` API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// The error message
    pub message: String,
    /// The type of error
    #[serde(rename = "type")]
    pub error_type: String,
    /// The parameter that caused the error, if any
    pub param: Option<String>,
    /// The error code, if any
    pub code: Option<String>,
}

impl LLMRequest for ChatCompletionRequest {
    fn messages(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.messages)?)
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

    fn to_map(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut map = HashMap::new();
        map.insert(
            "messages".to_string(),
            serde_json::to_value(&self.messages)?,
        );
        map.insert("model".to_string(), serde_json::to_value(&self.model)?);
        map.insert("stream".to_string(), serde_json::to_value(self.stream)?);
        if let Some(max_tokens) = self.max_tokens {
            map.insert("max_tokens".to_string(), serde_json::to_value(max_tokens)?);
        }
        if let Some(temperature) = self.temperature {
            map.insert(
                "temperature".to_string(),
                serde_json::to_value(temperature)?,
            );
        }
        if let Some(functions) = &self.functions {
            map.insert("functions".to_string(), serde_json::to_value(functions)?);
        }
        // Add any additional parameters
        for (key, value) in &self.additional_params {
            map.insert(key.clone(), value.clone());
        }
        Ok(map)
    }

    fn to_value(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn to_bytes(&self) -> Result<bytes::Bytes> {
        Ok(bytes::Bytes::from(serde_json::to_string(self)?))
    }
}
