use serde::{Deserialize, Serialize};

/// Represents the format of the response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseFormat {
    /// Must be one of `text` or `json_object`.
    pub r#type: String, // "text" or "json_object"
}

/// Represents usage statistics for the completion request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompletionUsage {
    /// Number of tokens in the generated completion.
    pub completion_tokens: u32,
    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// Total number of tokens used in the request (prompt + completion).
    pub total_tokens: u32,
}

/// Represents log probability information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogProbs {
    /// A list of message content tokens with log probability information.
    pub content: Option<Vec<TokenLogProb>>,
}

/// Represents a single token's log probability information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenLogProb {
    /// The token.
    pub token: String,
    /// The log probability of this token.
    pub logprob: f64,
    /// A list of integers representing the UTF-8 bytes representation of the token.
    pub bytes: Option<Vec<u8>>,
    /// List of the most likely tokens and their log probability, at this token position.
    pub top_logprobs: Vec<TopLogProb>,
}

/// Represents one of the most likely tokens and its log probability.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopLogProb {
    /// The token.
    pub token: String,
    /// The log probability of this token.
    pub logprob: f64,
    /// A list of integers representing the UTF-8 bytes representation of the token.
    pub bytes: Option<Vec<u8>>,
}
