#!/bin/bash
set -e

echo "Building slop-wasi-demo for wasm32-wasip1..."

# Check if the target is installed
if ! rustup target list --installed | grep -q wasm32-wasip1; then
    echo "Installing wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
fi

# Build from workspace root
cd "$(git rev-parse --show-toplevel)"

echo "Building release binary..."
cargo build --package slop-wasi-demo --target wasm32-wasip1 --release

echo "Copying WASM binary to demo directory..."
cp target/wasm32-wasip1/release/slop-wasi-demo.wasm \
   crates/slop-wasi-demo/

# Get file size
SIZE=$(wc -c < crates/slop-wasi-demo/slop-wasi-demo.wasm)
SIZE_KB=$((SIZE / 1024))

echo "✓ Build complete! WASM binary: ${SIZE_KB} KB"
echo ""
echo "Try it with:"
echo "  cd crates/slop-wasi-demo"
echo "  deno run --allow-read --allow-env run-wasm.ts example.json"
echo ""

# Check if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "wasm-opt is available. Run this to optimize:"
    echo "  wasm-opt -Oz -o slop-wasi-demo.opt.wasm slop-wasi-demo.wasm"
    echo "  mv slop-wasi-demo.opt.wasm slop-wasi-demo.wasm"
else
    echo "Tip: Install wasm-opt (binaryen) to reduce binary size by 30-50%"
fi
