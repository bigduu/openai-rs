//! Contains type definitions specific to the OpenAI API,
//! particularly for chat completions and streaming responses.

pub mod chat;
pub mod common;
pub mod function;
pub mod stream;

pub use chat::*;
pub use common::*;
pub use function::*;
pub use stream::*;
