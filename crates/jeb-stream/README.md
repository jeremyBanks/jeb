# jeb-stream

Async streaming utilities for Rust, built on top of `futures` and `tokio`.

## Overview

jeb-stream provides a composable streaming abstraction with:
- **Sources**: Read from stdin, files, or memory
- **Transforms**: Hex encode/decode, binary encode, line splitting
- **Sinks**: Write to stdout, files
- **Error handling**: Split ok/err streams, fail-fast, collect errors

## Features

- `stdio` - Enable stdin/stdout sources and sinks
- `fs` - Enable file system sources and sinks

## Usage

### Basic Example

```rust
use jeb_stream::{stdin, to_hex, stdout};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Read stdin, hex encode, write to stdout
    stdin()
        .pipe(to_hex())
        .pipe(stdout())
        .collect()
        .await;
}
```

### Working with Items

The `Item` enum represents stream data:

```rust
pub enum Item {
    Bytes(Bytes),  // Binary data
    Text(Arc<str>), // Text data
}
```

### Transform Functions

- `to_hex()` - Convert bytes/text to hex representation
- `to_binary()` - Convert items to binary format
- `lines()` - Split bytes into lines
- `from_hex()` - Decode hex to bytes

### Error Handling

```rust
use jeb_stream::{oks_and_errs, oks, errs, fail_fast};

// Split a stream into ok and error streams
let (ok_stream, err_stream) = oks_and_errs(input_stream);

// Collect only successful items
let results: Vec<Item> = oks(input_stream).collect().await;

// Stop on first error
let results = fail_fast(input_stream).collect().await;
```

## Testing

```bash
cargo test
```

98 tests verify transforms, sources, sinks, and error handling.

## Design

jeb-stream emphasizes:
- **Composability**: Small functions that chain together
- **Async-first**: Built on `futures::Stream` and `tokio`
- **Error handling**: Errors are stream items, not panics
- **Memory efficiency**: 64KB buffer limit for streaming
