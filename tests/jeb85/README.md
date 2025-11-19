# JEB85 Test Suite

Test suite for the minimal JEB85 encoder/decoder with chunking support.

## Test Structure

```
tests/jeb85/
├── inputs/          # Test input files
├── expected/        # Expected output files (for deterministic tests)
├── scripts/         # Test scripts
└── README.md        # This file
```

## Test Cases

### 01-encode-decode-z85.sh
**Goal**: Verify basic Z85 encode/decode roundtrip
**Input**: "Hello, World!" (13 bytes of ASCII)
**Pipeline**: `encode-z85 | decode-z85`
**Expected**: Should get back exactly the input

### 02-encode-z85-hello.sh
**Goal**: Verify deterministic Z85 encoding
**Input**: "Hello, World!"
**Pipeline**: `encode-z85`
**Expected**: Specific Z85-encoded output (to be determined)

### 03-chunked-encode-decode.sh
**Goal**: Verify chunked encode/decode roundtrip
**Input**: 100 KiB of zeros (should create 2 chunks: 64k + 36k)
**Pipeline**: `split-64k encode-z85 join-lines | split-lines decode-z85 join`
**Expected**: Should roundtrip perfectly

### 04-chunked-line-count.sh
**Goal**: Verify chunk splitting produces correct number of lines
**Input**: 100 KiB of zeros
**Pipeline**: `split-64k encode-z85 join-lines`
**Expected**: Exactly 2 lines (64k chunk + 36k chunk)

### 05-empty-input.sh
**Goal**: Verify empty input handling
**Input**: 0 bytes
**Pipeline**: `encode-z85 | decode-z85`
**Expected**: Empty output

### 06-error-no-join.sh
**Goal**: Verify error when stream-of-streams isn't joined
**Input**: Any file
**Pipeline**: `split-64k encode-z85` (no join!)
**Expected**: Should exit with error

### 07-exactly-64k.sh
**Goal**: Edge case - exactly 64 KiB input
**Input**: 64 KiB of zeros
**Pipeline**: `split-64k encode-z85 join-lines`
**Expected**: Exactly 1 line, and should roundtrip

### 08-binary-data-z85.sh
**Goal**: Verify binary (non-ASCII) data handling
**Input**: 16 bytes of 0x00-0x0f
**Pipeline**: `encode-z85 | decode-z85`
**Expected**: Should roundtrip perfectly

### 09-output-is-text.sh
**Goal**: Verify all encoded output is text-safe (no control characters)
**Input**: Binary data
**Pipeline**: `encode-z85`
**Expected**: Output should be valid UTF-8 with only printable ASCII characters

### 10-mixed-binary-text.sh
**Goal**: Verify mixed binary and text input handling
**Input**: "Hello\x00\x01\x02World\x00\xff" (text with embedded binary)
**Pipeline**: `encode-z85 | decode-z85`
**Expected**: Should roundtrip perfectly and produce text-safe encoded output

### 11-double-encode.sh
**Goal**: Verify double encoding/decoding (encoding encoded data)
**Input**: Mixed binary/text
**Pipeline**: `(split-64k encode-z85 join-lines) × 2` then `(split-lines decode-z85 join) × 2`
**Expected**: Should roundtrip perfectly through double encoding and remain text-safe

## Running Tests

```bash
# Run all tests
cd tests/jeb85/scripts
./run-all.sh

# Run individual test
./01-encode-decode-z85.sh
```

## Commands Being Tested

- `encode-z85` - Encode binary to Z85 text (pure base85)
- `decode-z85` - Decode Z85 text to binary
- `split-64k` - Split single stream into 64 KiB chunks (stream-of-streams)
- `join-lines` - Join stream-of-streams with newlines into single stream
- `split-lines` - Split single stream by newlines into stream-of-streams
- `join` - Join stream-of-streams with no delimiter into single stream

## Design Notes

### Output Format
- **All encoded outputs are text-only**: No control characters except newlines
- **Git-friendly**: Encoded outputs can be committed and diffed
- **Valid UTF-8**: All outputs should be valid UTF-8
- **Printable ASCII**: Z85 uses only printable ASCII characters (0x20-0x7E)

### Stream Types
- **Single stream**: Binary or text data
- **Stream-of-streams**: Multiple sub-streams

### Encoding/Decoding Rules
- When encode/decode encounters single stream: process it
- When encode/decode encounters stream-of-streams: map over each sub-stream

### Error Handling
- If pipeline ends with stream-of-streams (no join), it's an error
- Output must always be a single binary stream

### Chunking
- `split-64k` splits at 64 KiB boundaries (65,536 bytes)
- Last chunk may be smaller if input isn't multiple of 64k
- Empty input produces empty output (not an error)
