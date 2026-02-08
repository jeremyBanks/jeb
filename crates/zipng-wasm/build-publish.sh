#!/bin/bash
set -e

echo "Building zipng-wasm packages for publication..."

# Build WASM for all targets
echo ""
echo "==> Building WASM..."
./build.sh

# Update package.json files with publication metadata
echo ""
echo "==> Updating package metadata..."

# Node.js package - add bin field and better metadata
cat > pkg/node/package.json << 'EOF'
{
  "name": "zipng-wasm",
  "version": "0.1.0",
  "description": "Create PNG+ZIP polyglot files in WebAssembly - works in browsers, Node.js, and Deno",
  "keywords": ["wasm", "zip", "png", "polyglot", "image", "archive", "steganography"],
  "author": "Jeremy Banks <_@jeremy.ca>",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/jeremyBanks/zipng"
  },
  "homepage": "https://github.com/jeremyBanks/zipng#readme",
  "bugs": {
    "url": "https://github.com/jeremyBanks/zipng/issues"
  },
  "main": "zipng_wasm.js",
  "types": "zipng_wasm.d.ts",
  "bin": {
    "zipng": "./cli.mjs"
  },
  "files": [
    "zipng_wasm_bg.wasm",
    "zipng_wasm.js",
    "zipng_wasm.d.ts",
    "cli.mjs",
    "README.md"
  ],
  "engines": {
    "node": ">=14.0.0"
  }
}
EOF

# Web package - library only, no CLI
cat > pkg/web/package.json << 'EOF'
{
  "name": "zipng-wasm",
  "version": "0.1.0",
  "type": "module",
  "description": "Create PNG+ZIP polyglot files in WebAssembly - works in browsers, Node.js, and Deno",
  "keywords": ["wasm", "zip", "png", "polyglot", "image", "archive", "steganography", "browser"],
  "author": "Jeremy Banks <_@jeremy.ca>",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/jeremyBanks/zipng"
  },
  "homepage": "https://github.com/jeremyBanks/zipng#readme",
  "bugs": {
    "url": "https://github.com/jeremyBanks/zipng/issues"
  },
  "main": "zipng_wasm.js",
  "types": "zipng_wasm.d.ts",
  "files": [
    "zipng_wasm_bg.wasm",
    "zipng_wasm.js",
    "zipng_wasm.d.ts",
    "README.md"
  ],
  "sideEffects": [
    "./snippets/*"
  ]
}
EOF

# Bundler package
cat > pkg/bundler/package.json << 'EOF'
{
  "name": "zipng-wasm",
  "version": "0.1.0",
  "description": "Create PNG+ZIP polyglot files in WebAssembly - works in browsers, Node.js, and Deno",
  "keywords": ["wasm", "zip", "png", "polyglot", "image", "archive", "steganography", "webpack", "rollup", "vite"],
  "author": "Jeremy Banks <_@jeremy.ca>",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/jeremyBanks/zipng"
  },
  "homepage": "https://github.com/jeremyBanks/zipng#readme",
  "bugs": {
    "url": "https://github.com/jeremyBanks/zipng/issues"
  },
  "main": "zipng_wasm.js",
  "types": "zipng_wasm.d.ts",
  "files": [
    "zipng_wasm_bg.wasm",
    "zipng_wasm.js",
    "zipng_wasm.d.ts",
    "README.md"
  ],
  "sideEffects": false
}
EOF

# Copy CLI to node package and make it executable
echo ""
echo "==> Adding CLI to Node.js package..."
cp cli.mjs pkg/node/
chmod +x pkg/node/cli.mjs

# Copy README to all packages
echo ""
echo "==> Copying README to packages..."
for dir in pkg/node pkg/web pkg/bundler; do
  cp README.md "$dir/"
done

echo ""
echo "==> Package preparation complete!"
echo ""
echo "Packages ready for publication:"
echo "  - NPM (with CLI):  pkg/node/"
echo "  - NPM (web):       pkg/web/"
echo "  - NPM (bundler):   pkg/bundler/"
echo "  - JSR (Deno):      Use jsr.json in this directory"
echo ""
echo "To publish to NPM:"
echo "  cd pkg/node && npm publish"
echo ""
echo "To publish to JSR:"
echo "  deno publish --config jsr.json"
echo ""
echo "To test locally:"
echo "  cd pkg/node && npm link"
echo "  zipng -o test.png file.txt"
