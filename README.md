# OpenAI-RS

A Rust-based implementation of the OpenAI API client and server. This project provides a robust and efficient way to interact with OpenAI's services using Rust, with a focus on streaming support and flexible configuration.

## Project Structure

The project is organized into two main components:

- `core`: Contains the core OpenAI API client implementation with streaming support
- `server`: Provides a web server interface for the OpenAI API client

## Features

- Streaming support for real-time responses using Server-Sent Events (SSE)
- Flexible provider system for customizing:
  - HTTP client implementation
  - URL endpoints
  - API token management
  - Request parsing
  - Message processing
- Async/await support for efficient API calls
- JSON serialization/deserialization
- Comprehensive error handling
- Logging capabilities
- Web server interface

## Prerequisites

- Rust (latest stable version)
- Cargo (Rust's package manager)
- OpenAI API key

## Installation

1. Clone the repository:

    ```bash
    git clone https://github.com/yourusername/openai-rs.git
    cd openai-rs
    ```

2. Build the project:

```bash
cargo build
```

## Configuration

Before running the server, you need to set up your OpenAI API key. You can do this in several ways:

1. Environment variable:

    ```bash
    export OPENAI_API_KEY=your_api_key_here
    ```

2. Or use the `StreamingProxyContextBuilder` to configure your own token provider:

```rust
use core::context::StreamingProxyContextBuilder;
use core::token_provider::StaticTokenProvider;
use std::sync::Arc;

let context = StreamingProxyContextBuilder::new()
    .with_token_provider(Arc::new(StaticTokenProvider::new("your-api-key".to_string())))
    .build();
```

## Usage

### Running the Server

To start the server:

```bash
cargo run -p server
```

The server will start on the default port (check the server configuration for the exact port).

### Using the Core Client

You can use the core client in your Rust projects by adding it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
openai-rs-core = { path = "path/to/openai-rs/core" }
```

Example usage with custom configuration:

```rust
use core::context::StreamingProxyContextBuilder;
use core::token_provider::StaticTokenProvider;
use core::url_provider::StaticUrlProvider;
use std::sync::Arc;

// Create a custom context
let context = StreamingProxyContextBuilder::new()
    .with_token_provider(Arc::new(StaticTokenProvider::new("your-api-key".to_string())))
    .with_url_provider(Arc::new(StaticUrlProvider::new("https://api.openai.com/v1/chat/completions".to_string())))
    .build();

// Process a request
let response_stream = context.process_request(request_body).await?;
```

## Dependencies

The project uses several key dependencies:

- `tokio`: For async runtime
- `reqwest`: For HTTP client functionality
- `serde`: For JSON serialization/deserialization
- `actix-web`: For the web server
- `tracing`: For logging
- `bytes`: For efficient byte handling
- `futures`: For stream handling
- `anyhow`: For error handling

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
