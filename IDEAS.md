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

## Output Options

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
- Serialize all nodes which do not have non-text children
- Include tag names and parent information with special naming scheme:
  - `""` (empty string): The node's tag name
  - `"-"`: Parent tag name
  - `"--"`: Grandparent tag name (and so on)
  - `"-attribute-name"`: Parent's attribute values (for ALL attributes)
  - `"--attribute-name"`: Grandparent's attribute values (and so on)
  - Example: `"-id"` for parent's id attribute, `"--class"` for grandparent's class attribute
- Virtual attributes (always present in data model):
  - `@text`: Text node as first child of the node (empty string `""` for self-closing tags, `null` for tags with no text)
  - `@tail`: Text node following the node
  - `@index`: Sibling index for distinguishing identical adjacent parents
  - CDATA sections are parsed as regular text content (semantically equivalent)
  - Inherited from ancestors: `"-@text"`, `"--@text"`, `"-@tail"`, `"-@index"`, etc.
  - Options to control filtering:
    - Text filtering: Include all, filter whitespace-only, or filter empty (default: include all)
    - Index filtering: Include all, or filter except when needed for disambiguation (default: filter except when needed)
- XML metadata preservation as special leaf nodes:
  - Comments, DTD, doctype, XML headers/declarations, processing instructions preserved as leaf nodes
  - The `""` key contains the entire source of these elements
  - Allows round-tripping of all XML metadata
- Ordering preservation:
  - Attribute order preserved (using ordered maps)
  - Entity order preserved via `@index`
- This scheme doesn't collide with valid XML names (which can't be empty or start with hyphens or `@`)
- Handling identical adjacent parents:
  - `@index` is always present in the data model (like `@text`/`@tail`)
  - Default option filters it out except when needed for disambiguation
  - When a child node and the next node have identical attributes, `@index` is preserved on both
  - Requires buffering one extra entity to look ahead
  - Ensures all identical siblings get indexed (including the first), not just subsequent ones
- Lossless transformation:
  - All parent attributes captured at all levels
  - Self-closing tag distinction via `@text` (`""` vs `null`)
  - Attribute and entity ordering preserved
  - Metadata (comments, processing instructions, DTD, etc.) preserved in raw form
  - Only accepted semantic equivalence: CDATA vs regular text
- Input only due to non-bijective nature with JSON
