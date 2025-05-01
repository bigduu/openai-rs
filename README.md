# LLM Proxy

A flexible and configurable proxy server for Large Language Model APIs, with support for request processing pipelines and multiple LLM providers. This proxy server acts as a middleware between your applications and various LLM providers, offering enhanced functionality, request processing, and unified interface.

## Features

- 🔄 **Configurable Request Routing**
  - Route requests to different LLM providers based on path patterns
  - Support for multiple LLM backends in a single server instance
  - Flexible configuration through TOML files

- 🌊 **Streaming Support**
  - Full support for streaming responses
  - Configurable per-route streaming settings
  - Efficient handling of both streaming and non-streaming requests

- 🔌 **Extensible Provider System**
  - Currently supports OpenAI's Chat API
  - Modular design for easy addition of new providers
  - Abstract interfaces for consistent provider integration

- 🔧 **Request Processing Pipeline**
  - Customizable request processors
  - Chain multiple processors for complex transformations
  - Built-in support for request enhancement and modification

- 🔐 **Security Features**
  - Secure token management through environment variables
  - CORS configuration support
  - Request validation and sanitization

- 📝 **Observability**
  - Comprehensive logging with tracing support
  - Detailed error handling and reporting
  - Request/response monitoring capabilities

## Documentation

- **[Architecture Overview](./docs/ARCHITECTURE.md)**: Detailed explanation of the system architecture, components, and data flows.
- **[Implementing Custom Providers](./docs/IMPLEMENTING_PROVIDERS.md)**: Step-by-step guide on how to add support for new LLM providers.
- **[Implementation Examples](./docs/examples.md)**: Practical code examples for various system components and features.

## Architecture

The project is structured into three main crates:

### llm-proxy-core

Core functionality and traits:

```text
┌─────────────┐     ┌──────────────┐     ┌────────────┐
│   Request   │ ──> │  Processing  │ ──> │    LLM     │
│   Parser    │     │   Pipeline   │     │   Client   │
└─────────────┘     └──────────────┘     └────────────┘
```

Key Components:

- `Pipeline`: Generic request processing pipeline
- `Processor`: Trait for request processors
- `LLMClient`: Trait for LLM providers
- `RequestParser`: Trait for parsing raw requests
- Common types and error handling

### llm-proxy-openai

OpenAI-specific implementation:

```text
┌──────────────┐     ┌───────────────┐     ┌────────────┐
│  OpenAI API  │ <── │  OpenAI Chat  │ <── │  Request   │
│   Client     │     │    Client     │     │  Pipeline  │
└──────────────┘     └───────────────┘     └────────────┘
```

Features:

- OpenAI Chat API integration
- Streaming response handling
- Request/response type definitions
- API client implementation

### llm-proxy-server

HTTP server and configuration:

```text
┌──────────┐    ┌─────────┐    ┌──────────┐
│  HTTP    │ -> │  Route  │ -> │ Provider  │
│ Server   │    │ Handler │    │ Pipeline  │
└──────────┘    └─────────┘    └──────────┘
```

Components:

- Actix-web based HTTP server
- TOML configuration parsing
- Route management
- Pipeline orchestration

## Quick Start

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/llm-proxy.git
   cd llm-proxy
   ```

2. Copy the example configuration:

   ```bash
   cp llm-proxy-server/config.example.toml config.toml
   ```

3. Set your OpenAI API key:

   ```bash
   export OPENAI_API_KEY=your-api-key-here
   ```

4. Run the server:

   ```bash
   cargo run -p llm-proxy-server
   ```

The server will start at `http://127.0.0.1:3000` by default.

## Configuration

The proxy server is configured through a TOML file. Here's a detailed explanation of each section:

### LLM Provider Configuration

```toml
[llm.openai_chat]
provider = "openai"      # Provider type
type = "chat"           # API type
base_url = "https://api.openai.com/v1"  # API endpoint
token_env = "OPENAI_API_KEY"  # Environment variable for API key
supports_streaming = true  # Whether streaming is supported
```

### Request Processor Configuration

```toml
[processor.enhance_query]
type = "openai_chat"    # Processor type
config_value = "gpt-4"  # Model to use
additional_config = { 
    system_prompt = "Enhance this query"  # Custom configuration
}
```

### Route Configuration

```toml
[[route]]
path_prefix = "/v1/chat/completions"  # URL path to match
target_llm = "openai_chat"           # LLM provider to use
processors = ["enhance_query"]        # Processors to apply
allow_streaming = true               # Allow streaming responses
allow_non_streaming = true          # Allow non-streaming responses
```

### Server Configuration

```toml
[server]
host = "127.0.0.1"
port = 3000
cors_allowed_origins = ["*"]  # CORS settings
```

## Development

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p llm-proxy-server
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p llm-proxy-core
```

### Logging

```bash
# Run with debug logging
RUST_LOG=debug cargo run -p llm-proxy-server

# Run with trace logging
RUST_LOG=trace cargo run -p llm-proxy-server
```

## Implementing Custom Components

See the [Implementing Custom Providers](./docs/IMPLEMENTING_PROVIDERS.md) guide for detailed instructions.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Create a pull request

Please ensure your code:

- Passes all tests
- Includes appropriate documentation
- Follows the project's code style
- Has no clippy warnings

## License

This project is licensed under the MIT License - see the LICENSE file for details.
