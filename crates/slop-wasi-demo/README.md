# slop-wasi-demo

A demonstration of how to build and distribute Rust CLI tools as WASM modules for Deno, using WASI (WebAssembly System Interface).

## What This Demonstrates

This crate shows WASI-compatible patterns for building CLI tools:

- ✅ **Single-threaded async runtime**: Uses `tokio` with `current_thread` flavor (available but not needed)
- ✅ **File I/O**: Uses `std::fs` for file operations (WASI doesn't support async file I/O)
- ✅ **stdin/stdout**: Uses `std::io` for synchronous stream operations
- ✅ **Cross-platform**: Single WASM binary works on any platform with WASI support

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

## Testing with Node.js

Node.js v13+ has built-in WASI support, making it easy to test your WASM binary:

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

This is useful for quick testing before deploying with Deno.

## Running with Deno

### Direct Execution

```bash
cd crates/slop-wasi-demo

# Read from stdin
echo '{"hello":"world"}' | deno run --allow-read --allow-env run-wasm.ts

# Read from file
echo '{"hello":"world"}' > test.json
deno run --allow-read --allow-env run-wasm.ts test.json

# Compact output
deno run --allow-read --allow-env run-wasm.ts test.json --compact
```

### Install as a Command

```bash
cd crates/slop-wasi-demo

# Install globally
deno install -A -n slop-wasi run-wasm.ts

# Now you can run it anywhere
echo '{"hello":"world"}' | slop-wasi
slop-wasi input.json
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

- **Universal compatibility**: One binary for all platforms (x64, ARM, Windows, macOS, Linux)
- **No compilation matrix**: Don't need to build for multiple targets
- **Sandboxed execution**: WASI provides security boundaries
- **Easy updates**: Just replace the WASM file
- **No system dependencies**: Everything is self-contained

### Disadvantages

- **Larger file size**: WASM binaries are typically larger than native
- **Startup overhead**: 10-50ms to compile and instantiate WASM
- **Slight performance penalty**: 5-50% slower than native (varies by workload)
- **Limited APIs**: No networking, threading, or subprocess spawning (WASI Preview 1)

For I/O-bound JSON processing tools like this, the performance difference is negligible.

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
