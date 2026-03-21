# Contributing to Argus

Thank you for your interest in contributing to Argus! This guide will help you get started.

## Table of Contents

- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

## Development Setup

### Prerequisites

- Rust 1.75 or newer
- Git
- Docker (optional, for Redis/MinIO testing)

### Setup Steps

1. **Fork and Clone**
   ```bash
   git clone https://github.com/dedsecrattle/argus.git
   cd argus
   ```

2. **Install Rust**
   ```bash
   # Install rustup if you don't have it
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install required components
   rustup component add rustfmt clippy
   ```

3. **Build the Project**
   ```bash
   cargo build --workspace
   ```

4. **Run Tests**
   ```bash
   cargo test --workspace --lib
   ```

5. **Install CLI for Development**
   ```bash
   cargo install --path crates/argus-cli
   ```

## How to Contribute

### Reporting Bugs

- Use the [GitHub issue tracker](https://github.com/dedsecrattle/argus/issues)
- Include:
  - Rust version (`rustc --version`)
  - Operating system
  - Minimal reproduction case
  - Backtrace if available

### Suggesting Features

- Open an issue with the "enhancement" label
- Describe the use case clearly
- Consider if it fits the project's scope

### Contributing Code

1. **Find an Issue**
   - Look for issues with "good first issue" or "help wanted" labels
   - Or create your own issue to discuss the change

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Write clear, idiomatic Rust code
   - Add tests for new functionality
   - Update documentation

4. **Test Your Changes**
   ```bash
   # Run all tests
   cargo test --workspace --all-features
   
   # Run clippy
   cargo clippy --workspace --all-features -- -D warnings
   
   # Check formatting
   cargo fmt --all -- --check
   ```

5. **Commit Changes**
   ```bash
   git commit -m "feat: add new feature description"
   ```

6. **Push and Open PR**
   ```bash
   git push origin feature/your-feature-name
   # Open pull request on GitHub
   ```

## Code Style

### Formatting

We use `rustfmt` for consistent formatting:

```bash
cargo fmt --all
```

### Linting

We use `clippy` for linting:

```bash
cargo clippy --workspace --all-features -- -D warnings
```

### Code Guidelines

- Use `async/await` for async code
- Prefer `anyhow::Result` for error handling
- Document public APIs with `///` doc comments
- Write tests for all public functions
- Use meaningful variable and function names
- Keep functions focused and small

### Example Code Style

```rust
use anyhow::Result;

/// Fetches a URL with retry logic
///
/// # Arguments
/// * `url` - The URL to fetch
/// * `max_retries` - Maximum number of retry attempts
///
/// # Returns
/// The response body as bytes
///
/// # Errors
/// Returns an error if all retries fail
pub async fn fetch_with_retry(url: &str, max_retries: u32) -> Result<Vec<u8>> {
    let mut retries = 0;
    
    loop {
        match fetch_once(url).await {
            Ok(body) => return Ok(body),
            Err(e) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(100 * retries)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_with_retry_success() {
        let result = fetch_with_retry("https://httpbin.org/json", 3).await;
        assert!(result.is_ok());
    }
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with all features
cargo test --workspace --all-features

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

- Unit tests go in the same module with `#[cfg(test)]`
- Integration tests go in `tests/` directory
- Use `tokio::test` for async tests
- Mock external dependencies in tests

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_function_name() {
        // Given
        let input = "test input";
        
        // When
        let result = function_under_test(input).await;
        
        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }
}
```

## Documentation

### Code Documentation

- All public items must have documentation
- Include examples for complex APIs
- Use `#[doc]` attribute for module-level docs

```rust
//! # Web Crawling Module
//!
//! This module provides utilities for web crawling with proper
//! rate limiting and error handling.
```

### README Documentation

Keep README.md focused on:
- Quick start guide
- Installation instructions
- Basic usage examples
- Links to detailed documentation

### API Documentation

API documentation is automatically generated and published to docs.rs when publishing to crates.io.

## Submitting Changes

### Pull Request Process

1. **Update Documentation**
   - README if needed
   - CHANGELOG.md for significant changes
   - Code comments

2. **Ensure CI Passes**
   - All tests must pass
   - Clippy must not have warnings
   - Code must be properly formatted

3. **Write Clear PR Description**
   - What changes are made
   - Why they are needed
   - How to test them
   - Any breaking changes

4. **Link Issues**
   - Reference related issues with `#123`
   - Close issues with `Closes #123`

### Release Process

Only maintainers should create releases:

1. Update version numbers in all Cargo.toml files
2. Update CHANGELOG.md
3. Create git tag
4. Publish to crates.io
5. Create GitHub release

## Architecture Overview

### Crate Structure

```
argus/
├── argus-common     # Shared types and utilities
├── argus-config     # Configuration management
├── argus-fetcher    # HTTP fetching with retry logic
├── argus-parser     # HTML and sitemap parsing
├── argus-robots     # Robots.txt parsing
├── argus-dedupe     # Content deduplication
├── argus-storage    # Storage backends
├── argus-frontier   # URL frontier implementations
├── argus-worker     # Worker implementation
└── argus-cli        # Command-line interface
```

### Key Concepts

- **Frontier**: URL queue with prioritization
- **Fetcher**: HTTP client with retry logic
- **Parser**: Extracts links and metadata
- **Deduplication**: Prevents duplicate content
- **Storage**: Persists crawled data
- **Worker**: Coordinates all components

## Performance Guidelines

- Use connection pooling for HTTP clients
- Implement proper backpressure
- Avoid blocking operations in async code
- Use streaming for large responses
- Profile with `cargo flamegraph` when needed

## Security Considerations

- Never commit API keys or secrets
- Validate all external input
- Use HTTPS by default
- Respect robots.txt
- Implement rate limiting

## Getting Help

- Check existing issues and discussions
- Ask questions in GitHub Discussions
- Join our Discord/Slack (link in README)
- Email maintainers at maintainers@argus-crawler.org

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

Thank you for contributing! 🎉
