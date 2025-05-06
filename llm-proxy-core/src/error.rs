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
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    PythonError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::ProcessError(msg) => write!(f, "Process error: {msg}"),
            Self::LLMError(msg) => write!(f, "LLM error: {msg}"),
            Self::PipelineError(msg) => write!(f, "Pipeline error: {msg}"),
            Self::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            Self::Other(e) => write!(f, "Error: {e}"),
            Self::JsonError(e) => write!(f, "JSON error: {e}"),
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::PythonError(e) => write!(f, "Python error: {e}"),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
