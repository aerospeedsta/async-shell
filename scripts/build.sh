#!/usr/bin/env bash
set -ex

# Usage: ./scripts/build.sh [target]
# If no target is specified, it builds for the host architecture.

TARGET=${1:-$(rustc -vV | grep host | awk '{print $2}')}
echo "🚀 Building async-shell for target: $TARGET"

HOST=$(rustc -vV | grep host | awk '{print $2}')

# 1. Build Node Artifacts
echo "📦 Building Node API (.node)..."
npm install

if [[ "$TARGET" != "$HOST" && ("$TARGET" == *"aarch64"* || "$TARGET" == *"musl"*) ]]; then
  echo "Using Zig linker for Node cross-compilation..."
  npx napi build --release --platform --target "$TARGET" --zig
else
  npx napi build --release --platform --target "$TARGET"
fi

# 2. Build Python Wheels
echo "🐍 Building Python Wheel (.whl)..."
# In CI, we usually use maturin directly. 
# We pass --features python to ensure PyO3 is compiled.
if [[ "$TARGET" != "$HOST" && ("$TARGET" == *"aarch64"* || "$TARGET" == *"musl"*) ]]; then
  echo "Using Zig linker for Python wheel cross-compilation..."
  uvx maturin build --release --out dist --target "$TARGET" --features python --zig
else
  uvx maturin build --release --out dist --target "$TARGET" --features python
fi

echo "✅ Build complete for $TARGET"
