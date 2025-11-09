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
