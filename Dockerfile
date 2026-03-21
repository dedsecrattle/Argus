# Placeholder Docker image for Argus
# This will be replaced with the actual build after all crates are published

FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 argus

WORKDIR /app

# Install Argus from crates.io once all packages are published
# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# ENV PATH="/root/.cargo/bin:${PATH}"
# RUN cargo install argus-cli

# For now, just show a message
RUN cat > /usr/local/bin/argus << 'EOF'
#!/bin/sh
echo "Argus Web Crawler v0.1.0"
echo ""
echo "This is a placeholder Docker image."
echo "The full image will be available after all crates are published to crates.io."
echo ""
echo "Installation will be available via:"
echo "  cargo install argus-cli"
echo ""
echo "For now, visit:"
echo "  - GitHub: https://github.com/dedsecrattle/argus"
echo "  - crates.io: https://crates.io/crates/argus-cli"
EOF
RUN chmod +x /usr/local/bin/argus

USER argus

ENTRYPOINT ["argus"]
