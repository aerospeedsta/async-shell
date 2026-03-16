#!/usr/bin/env bash
set -ex

# Usage: ./scripts/build.sh [target]
# If no target is specified, it builds for the host architecture.

TARGET=${1:-$(rustc -vV | grep host | awk '{print $2}')}
echo "🚀 Building async-shell for target: $TARGET"

# 1. Build Node Artifacts
echo "📦 Building Node API (.node)..."
npm install
npx napi build --release --platform --target "$TARGET"

# 2. Build Python Wheels
echo "🐍 Building Python Wheel (.whl)..."
# In CI, we usually use maturin directly. 
# We pass --features python to ensure PyO3 is compiled.
if [[ "$TARGET" == *"musl"* ]]; then
  echo "Using Zig linker for MUSL wheel build..."
  uvx maturin build --release --out dist --target "$TARGET" --features python --zig
else
  uvx maturin build --release --out dist --target "$TARGET" --features python
fi

echo "✅ Build complete for $TARGET"
