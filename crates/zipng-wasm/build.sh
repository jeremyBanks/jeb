#!/bin/bash
set -e

echo "Building zipng-wasm for multiple targets..."

echo "Building for web target..."
wasm-pack build --target web --out-dir pkg/web --release

echo "Building for nodejs target..."
wasm-pack build --target nodejs --out-dir pkg/node --release

echo "Building for bundler target..."
wasm-pack build --target bundler --out-dir pkg/bundler --release

echo "Build complete! Outputs:"
echo "  - Web:     crates/zipng-wasm/pkg/web/"
echo "  - Node.js: crates/zipng-wasm/pkg/node/"
echo "  - Bundler: crates/zipng-wasm/pkg/bundler/"
