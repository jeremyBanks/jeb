# jeb (JSON Entity Bucket)

> **⚠️ DISCLAIMER**: This is vibe-coded slop. The humans responsible would like none of the blame but all of the credit.
>
> Nothing in this README or any other documents or files in this repository should currently be taken as necessarily true, accurate, or correct. The project version will remain 0.0.x for as long as this is the case.

A flexible command-line tool for merging, formatting, and searching JSON data.

## Features

- **Flexible Input Formats**: Handles JSON lines (newline-delimited), JSON arrays, and concatenated JSON objects
- **Multiple Input Sources**: Merge JSON from multiple files into a single output
- **Stream Processing**: Efficiently processes JSON by scanning for objects rather than requiring strict formatting
- **Standard I/O Support**: Use `-` for stdin/stdout in Unix pipelines

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/jeb`.

## Usage

### Basic Usage

```bash
# Read from stdin, write to stdout
echo '{"id": 1}' | jeb

# Process a single file (reads and writes to same file)
jeb data.json

# Process multiple files, merge into first file
jeb output.json input1.json input2.json

# Use explicit --from and --to
jeb --from input1.json --from input2.json --to output.json

# Read from stdin, write to file
jeb --to output.json < input.json

# Read from file, write to stdout
jeb --from input.json
```

### Input Format Handling

jeb automatically handles various JSON formats:

**JSON Lines (newline-delimited):**
```json
{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
```

**JSON Array:**
```json
[
  {"id": 1, "name": "Alice"},
  {"id": 2, "name": "Bob"}
]
```

**Concatenated JSON Objects:**
```json
{"id": 1}{"id": 2}{"id": 3}
```

**Mixed with arbitrary text:**
```text
Some preamble
{"id": 1, "name": "Alice"}
Random text
{"id": 2, "name": "Bob"}
```

### Output Format

jeb outputs JSON as a **JSON array** (line-by-line friendly):

```json
[{"id":1,"name":"Alice"}
,{"id":2,"name":"Bob"}
]
```

This format is both valid JSON and line-processable (skip the first character of each line).

### Options

- `-d, --debug`: Enable debug logging
- `-f, --from <FILE>`: Input file(s) (can be specified multiple times)
- `-t, --to <FILE>`: Output file
- `-h, --help`: Print help information

## Examples

### Merge multiple JSON files

```bash
jeb --from users.json --from admins.json --to all_users.json
```

### Format inconsistent JSON

```bash
# Input: mixed format JSON
cat messy.json
{"id":1}{"id":2}
[{"id":3}]

# Output: clean JSON lines
jeb messy.json
{"id":1}
{"id":2}
{"id":3}
```

### Use in a pipeline

```bash
# Extract JSON from API response and format
curl https://api.example.com/data | jeb | jq '.id'
```

## Architecture

### Library (`src/lib.rs`)

- **`parse_json_stream(input: &str)`**: Streaming JSON parser that extracts objects from any format
- **`extract_json_object(input: &str)`**: Low-level function to extract a single JSON object with proper brace matching
- Comprehensive test suite covering all input formats

### Binary (`src/main.rs`)

- Built on Tokio async runtime for efficient I/O
- CLI argument parsing with clap
- Async file I/O handling with stdin/stdout support
- Integration of parsing and output logic
- Safe in-place file modification with atomic rename

## Testing

Run the test suite:

```bash
cargo test
```

The tests cover:
- JSON lines format
- JSON array format
- Concatenated JSON objects
- Mixed formats with arbitrary text
- Nested objects
- Escaped characters in strings
- Edge cases (empty input, no JSON objects)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
