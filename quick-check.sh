#!/bin/bash

# Quick pre-commit check script
# Usage: ./quick-check.sh

echo "🚀 Quick Pre-Commit Check"
echo "========================="

# Run tests
echo "🧪 Running tests..."
if cargo test --quiet; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi

# Build WASM
echo "🔨 Building WASM..."
if cargo build --target wasm32v1-none --release --quiet; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Ready to commit!"
