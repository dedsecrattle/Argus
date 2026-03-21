#!/bin/bash

# Continue publishing remaining crates after rate limit

echo "🚀 Continuing Argus Publishing"
echo "============================="
echo ""
echo "Already published:"
echo "✅ argus-common v0.1.0"
echo "✅ argus-config v0.1.0"
echo "✅ argus-robots v0.1.0"
echo "✅ argus-dedupe v0.1.0"
echo "✅ argus-storage v0.1.0"
echo "✅ argus-fetcher v0.1.0"
echo ""
echo "Remaining to publish:"
echo "⏳ argus-parser v0.1.0"
echo "⏳ argus-frontier v0.1.0"
echo "⏳ argus-worker v0.1.0"
echo "⏳ argus-cli v0.1.0"
echo ""

# Function to publish with retry
publish_with_retry() {
    local crate=$1
    echo "📦 Publishing $crate..."
    
    if cargo publish -p $crate; then
        echo "✅ Published $crate"
        sleep 30  # Wait between publishes
        return 0
    else
        echo "❌ Failed to publish $crate (rate limited?)"
        return 1
    fi
}

# Try to publish remaining crates
publish_with_retry "argus-parser" && \
publish_with_retry "argus-frontier" && \
publish_with_retry "argus-worker" && \
publish_with_retry "argus-cli"

echo ""
echo "🎉 Publishing complete!"
echo ""
echo "Next steps:"
echo "1. Create GitHub release: https://github.com/dedsecrattle/argus/releases/new"
echo "2. Build Docker image: docker build -t dedsecrattle/argus:v0.1.0 ."
echo "3. Push to Docker Hub: docker push dedsecrattle/argus:v0.1.0"
