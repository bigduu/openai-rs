# OpenAI-RS

A Rust-based implementation of the OpenAI API client and server. This project provides a robust and efficient way to interact with OpenAI's services using Rust.

## Project Structure

The project is organized into two main components:

- `core`: Contains the core OpenAI API client implementation
- `server`: Provides a web server interface for the OpenAI API client

## Features

- Async/await support for efficient API calls
- Streaming support for real-time responses
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

Before running the server, you need to set up your OpenAI API key as an environment variable:

```bash
export OPENAI_API_KEY=your_api_key_here
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

## Dependencies

The project uses several key dependencies:

- `tokio`: For async runtime
- `reqwest`: For HTTP client functionality
- `serde`: For JSON serialization/deserialization
- `actix-web`: For the web server
- `tracing`: For logging

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
