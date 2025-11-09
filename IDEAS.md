# Future Ideas and Enhancements

> **IMPORTANT**: This document contains brainstorming and potential ideas for future development. Nothing listed here represents a commitment to implement these features. These are exploratory concepts that may or may not be pursued.

## Permissive JSON Parsing

### Comment Support
- Line comments: Both `//` (JavaScript-style) and `#` (shell-style)
- Block comments: `/* ... */` (C-style)

### Relaxed Syntax
- Treat `:` and `,` as optional/equivalent to whitespace
- Allow more flexible delimiter usage for easier hand-editing

### Alternative String Delimiters
- Support single quotes (`'string'`) in addition to double quotes
- Support backticks (`` `string` ``) for template-style strings
- Allow multiline strings without escaping newlines

### Relaxed Key Requirements
- Unquoted or single-quoted object keys

## Data Operations

### Filtering
- Add query support to filter objects (e.g., `--filter 'id > 5'`)

### Field Selection
- Project specific fields (e.g., `--select id,name`)

### Sorting
- Sort objects by field values

### Deduplication
- Remove duplicate objects based on key fields

### JSON Patch/Merge
- Apply transformations to objects

## Output Options

### Pretty Printing
- Option to output formatted JSON instead of JSON lines

### Streaming Output
- Write objects as they're parsed for large files

## Validation and Analysis

### Schema Validation
- Validate objects against JSON Schema

### Statistics
- Report on object counts, field distributions, etc.

## Pipeline Definition and Visualization

### Data-Driven Pipeline Configuration
- Define data flow using data/configuration instead of code
- CLI could construct pipeline definitions from arguments
- Visualize the data flow pipeline
- Show how transformations connect and compose

## Alternative Input Formats

> **Note on Format Philosophy**: JSON is canonical for jeb (hence the name). Both XML and bencoding are not fully bijective with JSON, so they are primarily input formats to get data into the canonical JSON representation.

### JEB Binary Encodings
- Use no prefix for simple values that don't require encoding and don't use special characters
- Need to investigate current implementation to verify this optimization

### Bencode Support (Input Only)
- Deserialize bencoded data (BitTorrent encoding format)
- Input only initially due to non-bijective nature with JSON
- Allows importing data from torrent files and similar sources

### XML Support (Input Only)
- Serialize nodes that don't have children (self-closing or leaf nodes)
- Include tag names and parent information with special naming to avoid collisions:
  - `xml-name`: The node's tag name
  - `xml-parent-xml-name`: Parent tag name
  - `xml-parent-id`: Parent tag's id attribute value
- Note: Naming scheme is intentionally verbose to avoid collisions with actual XML content
- Starting point that could be refined with a cleaner approach later
- Input only due to non-bijective nature with JSON
