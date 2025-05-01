use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Error during request parsing
    ParseError(String),
    /// Error during request processing
    ProcessError(String),
    /// Error communicating with LLM service
    LLMError(String),
    /// Error in pipeline execution
    PipelineError(String),
    /// Configuration error
    ConfigError(String),
    /// Generic error wrapper
    Other(anyhow::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Error::ProcessError(msg) => write!(f, "Process error: {msg}"),
            Error::LLMError(msg) => write!(f, "LLM error: {msg}"),
            Error::PipelineError(msg) => write!(f, "Pipeline error: {msg}"),
            Error::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            Error::Other(e) => write!(f, "Error: {e}"),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::ParseError(err.to_string())
    }
}
