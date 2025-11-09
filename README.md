# JEB (JSON Entity Bucket)

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

JEB automatically handles various JSON formats:

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
```
Some preamble
{"id": 1, "name": "Alice"}
Random text
{"id": 2, "name": "Bob"}
```

### Output Format

JEB always outputs JSON as **JSON Lines** (newline-delimited JSON):

```json
{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
```

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

- CLI argument parsing with clap
- File I/O handling with stdin/stdout support
- Integration of parsing and output logic

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

## Future Enhancements

- **JSON Comment Support**: Handle comments in JSON input (e.g., `// comment` and `/* comment */`)
- **Filtering**: Add query support to filter objects (e.g., `--filter 'id > 5'`)
- **Field Selection**: Project specific fields (e.g., `--select id,name`)
- **Sorting**: Sort objects by field values
- **Deduplication**: Remove duplicate objects based on key fields
- **Pretty Printing**: Option to output formatted JSON instead of JSON lines
- **Streaming Output**: Write objects as they're parsed for large files
- **JSON Patch/Merge**: Apply transformations to objects
- **Schema Validation**: Validate objects against JSON Schema
- **Statistics**: Report on object counts, field distributions, etc.

## License

This project is open source.
