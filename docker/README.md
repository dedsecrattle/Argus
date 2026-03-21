# Docker Files for Argus

## Dockerfile (Current)
- **Purpose**: Placeholder image showing installation instructions
- **Status**: ✅ Working, pushed to Docker Hub
- **Use when**: You want to try Argus before full release

## Dockerfile.full
- **Purpose**: Complete image with JavaScript rendering support
- **Features**: Includes Chrome for SPA crawling
- **Status**: ⚠️ ARM64 compatibility issues
- **Use when**: JavaScript rendering is required (x86_64 only)

## Dockerfile.source
- **Purpose**: Build from source code
- **Features**: Latest code, all features
- **Status**: ⚠️ Requires all crates to be published
- **Use when**: Building from source after full release

## Building

```bash
# Current placeholder
docker build -t argus:placeholder .

# Full version (x86_64 only)
docker build -f Dockerfile.full -t argus:full .

# From source (after all crates published)
docker build -f Dockerfile.source -t argus:source .
```

## Future Plan

Once all crates are published:
1. Update Dockerfile to build from `cargo install argus-cli`
2. Add multi-architecture support (amd64, arm64)
3. Reduce image size with alpine-based variant
