# slop-wasi-demo

A demonstration of how to build and distribute Rust CLI tools as WASM modules
using WASI (WebAssembly System Interface).

**✅ Verified Working:** Node.js WASI, wasmtime **⚠️ Deno Status:** WASI support
is limited (see [Deno Compatibility](#deno-compatibility) below)

## What This Demonstrates

This crate shows WASI-compatible patterns for building CLI tools:

- ✅ **Single-threaded async runtime**: Uses `tokio` with `current_thread`
  flavor - **works with WASI!**
- ✅ **File I/O**: Uses `std::fs` for file operations (WASI doesn't support
  async file I/O)
- ✅ **stdin/stdout**: Uses `std::io` for synchronous stream operations
- ✅ **Cross-platform**: Single WASM binary works with any WASI-compatible
  runtime

## WASI Compatibility Notes

### What Works ✅

- `std::fs::File` - Synchronous file operations
- `std::io::stdin()` / `stdout()` - Synchronous stdio
- `tokio::time` - Timers and delays (with `rt` feature)
- `serde_json` - JSON parsing and serialization
- Command-line argument parsing
- Environment variables
- Single-threaded tokio runtime (`current_thread` flavor)

### What Doesn't Work ❌

- `tokio::fs` - Uses thread pools (not available in WASI)
- `tokio::io::stdin/stdout` - Not supported (use `std::io` instead)
- `tokio::net` - Network sockets (WASI Preview 1 limitation)
- Multi-threaded runtime - WASI is single-threaded only
- Process spawning - Not supported in WASI

## Building

### Prerequisites

1. Install the WASI target:
   ```bash
   rustup target add wasm32-wasip1
   ```

2. Install Deno (for running the WASM):
   ```bash
   curl -fsSL https://deno.land/install.sh | sh
   ```

### Build the WASM Binary

From the workspace root:

```bash
cargo build --package slop-wasi-demo --target wasm32-wasip1 --release
```

Copy the WASM binary to the demo directory:

```bash
cp target/wasm32-wasip1/release/slop-wasi-demo.wasm \
   crates/slop-wasi-demo/
```

### Optional: Optimize the WASM Binary

Install `wasm-opt` from [binaryen](https://github.com/WebAssembly/binaryen):

```bash
# On macOS
brew install binaryen

# On Linux
apt-get install binaryen
```

Optimize the binary:

```bash
wasm-opt -Oz -o slop-wasi-demo.opt.wasm slop-wasi-demo.wasm
mv slop-wasi-demo.opt.wasm slop-wasi-demo.wasm
```

This can reduce the WASM size by 30-50%.

## Deno Compatibility

**Current Status (Deno 2.5.6 as of Nov 2025):**

- `node:wasi` - Non-functional (stub implementation only)
- `std/wasi` - Deprecated, has compatibility issues with modern Rust/tokio

**Recommendation:** Use Node.js WASI or wasmtime for now. Deno's WASI support is
expected to improve with WASI 0.2 implementation
([Issue #24289](https://github.com/denoland/deno/issues/24289)).

You can still run the WASM via Deno using Node compatibility:

```bash
deno run --allow-read --allow-env test-wasm-node.mjs example.json
```

## Running with Node.js

Node.js v13+ has excellent built-in WASI support and works perfectly with this
demo:

```bash
cd crates/slop-wasi-demo

# Test with file input
node test-wasm-node.mjs example.json

# Test with stdin
echo '{"hello":"world"}' | node test-wasm-node.mjs

# Test compact mode
node test-wasm-node.mjs example.json --compact

# Test help
node test-wasm-node.mjs --help
```

## Distribution Options

### Option 1: Embed WASM in TypeScript

You can embed the WASM binary directly in your TypeScript file:

```typescript
const wasmBinary = new Uint8Array([
  // ... binary data here
]);
```

This creates a single-file executable that can be published to:

- [deno.land/x](https://deno.land/x)
- [JSR](https://jsr.io)

### Option 2: Download on Demand

Fetch the WASM binary from a URL:

```typescript
const response = await fetch("https://example.com/slop-wasi-demo.wasm");
const wasmBinary = new Uint8Array(await response.arrayBuffer());
```

### Option 3: Bundle with Deno

Package the WASM and TypeScript together and let users install with:

```bash
deno install -A https://example.com/run-wasm.ts
```

## Comparing to Native Binary

### Advantages of WASM Distribution

- **Universal compatibility**: One binary for all platforms (x64, ARM, Windows,
  macOS, Linux)
- **No compilation matrix**: Don't need to build for multiple targets
- **Sandboxed execution**: WASI provides security boundaries
- **Easy updates**: Just replace the WASM file
- **No system dependencies**: Everything is self-contained

### Disadvantages

- **Larger file size**: WASM binaries are typically larger than native
- **Startup overhead**: 10-50ms to compile and instantiate WASM
- **Slight performance penalty**: 5-50% slower than native (varies by workload)
- **Limited APIs**: No networking, threading, or subprocess spawning (WASI
  Preview 1)

For I/O-bound JSON processing tools like this, the performance difference is
negligible.

## Adapting Your Own CLI

To make your CLI WASI-compatible:

1. **Use single-threaded Tokio**:
   ```toml
   tokio = { version = "1.0", default-features = false, features = [
       "io-util",
       "io-std",
       "rt",
       "macros",
   ] }
   ```

2. **Change the runtime flavor**:
   ```rust
   #[tokio::main(flavor = "current_thread")]
   async fn main() { ... }
   ```

3. **Replace `tokio::fs` with `std::fs`**:
   ```rust
   // Before:
   let file = tokio::fs::File::open(path).await?;

   // After:
   let file = std::fs::File::open(path)?;
   ```

4. **Keep `tokio::io` for stdin/stdout**:
   ```rust
   let mut stdin = tokio::io::stdin();
   let mut stdout = tokio::io::stdout();
   ```

## Future: WASI Preview 2

WASI Preview 2 (WASI 0.2) provides:

- Component model support
- Better networking APIs
- Improved threading support

Once Deno supports WASI Preview 2, many current limitations will be lifted.

## License

Same as parent project (MIT OR Apache-2.0)
