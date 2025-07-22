#!/bin/bash
set -e

echo "🔧 Formatting code..."
cargo fmt

echo "📋 Running clippy..."
cargo clippy --lib --tests

echo "🧪 Running tests..."
cargo test

echo "📚 Checking docs..."
cargo doc --no-deps

echo "✅ Library checks complete!"