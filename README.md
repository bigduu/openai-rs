# LLM Proxy

A flexible and configurable proxy server for Large Language Model APIs, with support for request processing pipelines and multiple LLM providers.

## Features

- 🔄 Configurable request routing and processing pipelines
- 🌊 Support for both streaming and non-streaming responses
- 🔌 Extensible provider system (currently supporting OpenAI)
- 🔧 Custom request processors for enhancing or modifying requests
- 🔐 Secure token management through environment variables
- 📝 Comprehensive logging and error handling
- 🛠️ Easy configuration through TOML files

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

The proxy server is configured through a TOML file. See `llm-proxy-server/config.example.toml` for a complete example with comments.

### Key Configuration Sections:

- `[llm.<name>]`: Configure LLM backend services
- `[processor.<name>]`: Define request processors
- `[[route]]`: Set up request routing rules
- `[server]`: Configure server settings

Example configuration:
```toml
[llm.openai_chat]
provider = "openai"
type = "chat"
base_url = "https://api.openai.com/v1"
token_env = "OPENAI_API_KEY"
supports_streaming = true

[processor.enhance_query]
type = "openai_chat"
config_value = "gpt-4"
additional_config = { system_prompt = "Enhance this query" }

[[route]]
path_prefix = "/v1/chat/completions"
target_llm = "openai_chat"
processors = ["enhance_query"]
allow_streaming = true
allow_non_streaming = true
```

## Project Structure

- `llm-proxy-core`: Core traits and types
- `llm-proxy-openai`: OpenAI-specific implementations
- `llm-proxy-server`: HTTP server and configuration

## Adding New Providers

1. Create a new crate (e.g., `llm-proxy-anthropic`)
2. Implement the core traits from `llm-proxy-core`
3. Add a feature flag to `llm-proxy-server`

## Development

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=debug cargo run -p llm-proxy-server
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Create a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
