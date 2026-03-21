# Docker Setup for Argus

## Building the Docker Image

```bash
# Build the image
docker build -t dedsecrattle/argus:v0.1.0 .
docker build -t dedsecrattle/argus:latest .

# Test locally
docker run --rm dedsecrattle/argus:latest --version
```

## Pushing to Docker Hub

```bash
# Login to Docker Hub
docker login

# Push images
docker push dedsecrattle/argus:v0.1.0
docker push dedsecrattle/argus:latest
```

## Docker Compose Example

```yaml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  argus:
    image: dedsecrattle/argus:v0.1.0
    command: crawl --redis-url redis://redis:6379 --workers 4
    volumes:
      - ./data:/data
    depends_on:
      - redis
    environment:
      - RUST_LOG=info

volumes:
  redis_data:
```

## Usage Examples

### Basic Crawling
```bash
docker run -v $(pwd)/data:/data dedsecrattle/argus:latest \
  crawl --seed-url https://example.com --storage-dir /data
```

### Distributed Crawling
```bash
docker-compose up -d redis
docker run --link redis:redis dedsecrattle/argus:latest \
  crawl --redis-url redis://redis:6379 --workers 8
```

### With JavaScript Rendering
```bash
docker run dedsecrattle/argus:latest \
  crawl --seed-url https://spa-example.com --js-render
```

## Multi-Architecture Support

The Dockerfile supports multiple architectures. To build for all:

```bash
# Install buildx
docker buildx install

# Create builder
docker buildx create --use

# Build and push all architectures
docker buildx build --platform linux/amd64,linux/arm64 \
  -t dedsecrattle/argus:v0.1.0 \
  -t dedsecrattle/argus:latest \
  --push .
```

## Docker Hub Repository

https://hub.docker.com/r/dedsecrattle/argus
