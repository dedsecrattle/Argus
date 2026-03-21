FROM rust:1.75 as builder

# Install system dependencies for headless Chrome
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Chrome for JavaScript rendering
RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the CLI
RUN cargo build --release -p argus-cli

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 argus

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/argus /usr/local/bin/argus

# Create data directory
RUN mkdir -p /data && chown argus:argus /data

USER argus

ENTRYPOINT ["argus"]
