# Placeholder Docker image for Argus
# This will be replaced with the actual build after all crates are published

FROM debian:bullseye-slim

# Install runtime and build dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 argus

WORKDIR /app

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Argus from crates.io
RUN cargo install argus-crawler

USER argus

ENTRYPOINT ["argus"]
