FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install additional development tools
RUN apt-get update && apt-get install -y \
    lldb \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for development
RUN cargo install cargo-watch

# Set up the workspace
WORKDIR /workspace 