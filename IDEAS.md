# Future Ideas and Enhancements

> **IMPORTANT**: This document contains brainstorming and potential ideas for future development. Nothing listed here represents a commitment to implement these features. These are exploratory concepts that may or may not be pursued.

## Pipeline-Based CLI Model

### Overview

Redesign the CLI as a concise constructor DSL for building data flow graphs. Each node type represents a transformation or I/O operation with typed inputs/outputs.

### Data Flow Graph Model

- **Nodes**: Transformation operations (parse, merge, filter, etc.) or I/O operations (read file, stdin, stdout)
- **Edges**: Typed stream connections between nodes
- **Stream Types**: Each edge has a type (binary stream, JSON object stream, etc.)
- **Cardinality Constraints**: Each node type specifies min/max number of inputs and outputs
- **Defaults**: One binary stream input from stdin, one binary stream output to stdout

### CLI Argument Syntax

**Basic Principle**: Each argument specifies a new node in the graph.

**Node Naming**:
- Node names: Start with uppercase letter, contain uppercase letters, digits, underscores (e.g., `A1`, `FILE1`, `MERGED_DATA`)
- Command names: Start with lowercase letter (e.g., `parse-json`, `sort-keys`, `merge`)
- Implicit input nodes: Named `I1`, `I2`, `I3`, etc.
- Implicit operation nodes: Named `A1`, `A2`, `A3`, etc.
- Reserved names: `ALL`, `EACH` (for future use)

**Explicit Naming**: Append `-as-NAME` to give a node an explicit name
```bash
parse-json-as-PARSER1
```

**Explicit Connections**:
- Input connections: `-from-NODE1-NODE2-NODE3`
- Output connections: `-to-NODE1-NODE2-NODE3`

```bash
merge-from-A1-A2-A3-to-COMBINED
```

**Implicit Connection Rules**:
1. If a new node requires inputs and has no explicit `-from`:
   - **Single input required**: Connect from the most recent node with available output
   - **Multiple inputs accepted**: Connect from all previous nodes with available outputs
2. If no suitable inputs exist, implicitly create a stdin reader node (`I1`, `I2`, etc.)
3. After all nodes are created, if any outputs are unconnected:
   - Create an implicit merge node connected to all unconnected outputs
   - Connect merge to an implicit stdout writer

**Examples**:

Simple linear pipeline (implicit connections):
```bash
jeb parse-json sort-keys
# Creates: I1 (stdin) → A1 (parse-json) → A2 (sort-keys) → O1 (stdout)
```

Multiple inputs with implicit merge:
```bash
jeb read-file-as-FILE1 read-file-as-FILE2 parse-json-from-FILE1 parse-json-from-FILE2
# Creates: FILE1 → parse (A1) ┐
#          FILE2 → parse (A2) ┴→ merge (M1) → stdout (O1)
```

### Node Parameter Syntax

**Status**: Not yet defined. Need to catalog all node types and their parameter requirements to determine appropriate syntax.

Considerations:
- How to specify file paths, buffer sizes, sort specifications, filter expressions, etc.
- Need to distinguish node parameters from connection directives (`-as-NAME`, `-from-X`, `-to-Y`)
- Should be concise but unambiguous

**Example Node Type: SQLite**

The SQLite node demonstrates multiple parameter types:
```bash
jeb read-file:latest.json sqlite:foo.db where-prefix:(type:Entry)
```

Parameters:
- **Database path**: `sqlite:foo.db` (or empty string `sqlite:` for temporary file)
- **Mode flags**: replace vs extend existing database
- **Query/filter expressions**: `where-prefix:(type:Entry)`
- **Default behavior**: If no path specified, use `""` (temporary file created by SQLite)

The SQLite node has special output types (TBD) for query results.

### Named and Optional Outputs

Nodes can have multiple types of outputs:
- **Required outputs**: Referenced by just the node name (e.g., `NODE`)
- **Optional/named outputs**: Referenced with dot notation (e.g., `NODE.portname`)

When a node has both required and optional outputs, referring to it by name alone means its required outputs.

### Output Representations

The constructed graph can be:
1. **Executed**: Run the pipeline
2. **Serialized to JSON**: Output the graph definition for inspection or reuse
3. **Visualized**: Render a crude topological sort or graph representation (TBD)

### Default Option Values

**Potential feature**: A meta-command to set default values for options globally, which then applies transparently to all nodes that accept those option names.

Example:
```bash
jeb set-default:max:1024 \
    read:file1.json parse-json sort \
    read:file2.json parse-json sort \
    merge
# Both sort nodes would inherit max:1024
```

Or possibly:
```bash
jeb defaults(max:1024,strict:true) \
    parse-json sort \
    parse-json sort
# All applicable nodes inherit the defaults
```

This would avoid repeating common options across many nodes while passing through all other node aspects transparently.

### Open Questions

- How to handle multiple implicit input nodes sensibly
- Best syntax for node parameters (colon-separated? key=value? other?)
- Graph validation rules and error messages
- How to support node type discovery/documentation

### Potential JSON Schema Model

One possible approach for representing the pipeline model and CLI mapping:

**Option Syntax**: Using `:` sets the first unnamed option, or use `()` for named options
- `head:128` - sets first option to 128
- `head(max:128)` - explicitly names the option

**Example Schema**:
```json
{
  // TitleCamelCase
  "stream_types": {
    "Binary": {},
    "Json": {}
  },

  // lower-kebab-case
  "node_types": {
    "read": {
      "path": "/dev/stdin",
      "OUT": "Bytes"
    },

    "write": {
      "path": "/dev/stdout",
      "IN": "Bytes"
    },

    "sort": {
      "max": 512,
      "IN": "Json",
      "OUT": "Json"
    },

    // Merges multiple input streams into a single output stream,
    // attempting to maintain our sorting.
    "merge": {
      "descending": false,
      "IN": {
        "type": "json-objects",
        "plural": true
      },
      "OUT": {
        "type": "json-objects"
      }
    },

    // Concatenates multiple input streams (of the same type)
    // into a single output stream.
    "concat": {
      "IN": {
        "plural": true
      },
      "OUT": {
        "type": "IN"
      }
    },

    // Duplicates the input stream to multiple output streams.
    "tee": {
      "IN": {},
      "OUT": {
        "type": "IN",
        "plural": true
      }
    },

    // Parses a binary stream into a stream of JSON objects.
    "parse-json": {
      "strict": false,
      "IN": "Bytes",
      "OUT": "Json",
      "OUT.failed": "Bytes"
    },

    // Serializes a stream of JSON objects into a binary stream.
    "serialize-json": {
      "IN": "Json",
      "OUT": "Bytes"
    },

    // Computes cryptographic hash digests of binary data.
    // Outputs a JSON object with hash algorithm names as keys.
    "digests": {
      "algorithms": ["BLAKE3", "SHA1", "SHA-256", "SHA-384", "SHA-512", "SHA3"],
      "IN": "Bytes",
      "OUT": "Json"
    }
  },

  // UPPER_SNAKE_CASE
  "nodes": {
    "G1": {
      "type": "ReadPath",
      "path": "/dev/stdin"
    },

    "A1": {
      "type": "parse-json",
      "strict": true
    },

    "G2": {
      "type": "WritePath",
      "path": "/dev/stdout"
    }
  }
}
```

**Example Pipeline** (conceptual):
```bash
jeb \
  read:data.json parse-json \
  read:/dev/stdin parse-json \
  merge serialize-json \
  tee write:data.json \
      order:type_id,id sort(max:1024) head:128 write:/dev/stdout \
      order:name sort(max:1024) head:16 write:1.json
```

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

> **Note on naming**: The tool and library for JSON is **jeb** (lowercase). The text/binary encoding family is **JEB** (uppercase).

- JEB is a family of text encoding variations based on base64 and base85
- Attempts to preserve source as readable when it avoids problematic characters
- Use no prefix for simple values that don't require encoding and don't use special characters
- Current implementation uses block-based preservation (encode66/decode66 for URLs, encode92/decode92 for string literals)

**Internal Representation**: The canonical internal binary format for JSON is **Latin-1 passthrough** - the most generic and native option, even though it may be less efficient when encoded as JSON. JEB encodings (JEB64, etc.) are available as encoding options but are not used as the core internal representation.

**Potential Optimizations**:

1. **Full string passthrough**: If an entire string consists only of safe characters and contains no prefix markers (`~`, `|`, etc.), pass it through completely unencoded. This works as an optimization on top of the existing block-based approach, handling the common case of already-safe strings efficiently.
   - **Size limit**: This optimization only applies to strings within the 9999-block lookahead buffer size. For longer strings, we can't determine if they're entirely safe without reading past the buffer limit, so block-based encoding is used instead.

2. **Raw mode prefix**: A special prefix (e.g., `|~`) meaning "everything after this point is unencoded passthrough". This would be highly efficient for files with small encoded headers followed by large safe text bodies.
   - The prefix only applies at block transition points, so `|~` appearing naturally in raw content is not a concern
   - **Canonical encoding strategy**: Since 9999 is the maximum number of blocks, buffer the required number of bytes to look ahead that far. If the stream ends within that distance AND all remaining bytes are safe characters, use `|~` prefix instead of block encoding. This makes the encoding deterministic - same input always produces the same output - while keeping buffering costs bounded.

### Bencode Support (Input Only)
- Deserialize bencoded data (BitTorrent encoding format)
- Input only initially due to non-bijective nature with JSON
- Allows importing data from torrent files and similar sources

### Protocol Buffers Wire Format

Support for serialization and deserialization of Protocol Buffers wire format:
- Both encoding and decoding operations
- **Heuristic decoding approach**: Since wire format cannot inherently distinguish between binary data and nested messages without a schema, use best-effort auto-detection at each nesting level:
  - Check if data looks like valid UTF-8 (decode as string)
  - Check if data looks like JSON (parse as JSON)
  - Check if data looks like proto wire format (decode recursively)
  - Check if data looks like bencode (decode as bencode)
  - Otherwise treat as binary blob (base64 or similar representation)

### XML Support (Input Only)
- Serialize all nodes which do not have non-text children
- Include tag names and parent information with special naming scheme:
  - `""` (empty string): The node's tag name
  - `"-"`: Parent tag name
  - `"--"`: Grandparent tag name (and so on)
  - `"-attribute-name"`: Parent's attribute values (for ALL attributes)
  - `"--attribute-name"`: Grandparent's attribute values (and so on)
  - Example: `"-id"` for parent's id attribute, `"--class"` for grandparent's class attribute
- Attribute value handling:
  - Boolean attributes without values (HTML): converted to `true`
    - Example: `<input disabled>` → `"disabled": true`
  - Attributes with values: preserved as strings
  - Round-trip: `true` values output as valueless boolean attributes
- Virtual attributes (always present in data model):
  - `@text`: Text node as first child of the node (empty string `""` for self-closing tags, `null` for tags with no text)
  - `@tail`: Text node following the node
  - `@index`: Sibling index for distinguishing identical adjacent parents
  - Inherited from ancestors: `"-@text"`, `"--@text"`, `"-@tail"`, `"-@index"`, etc.
  - Options to control filtering:
    - Text filtering: Include all, filter whitespace-only, or filter empty (default: include all)
    - Index filtering: Include all, or filter except when needed for disambiguation (default: filter except when needed)
- XML metadata preservation as special leaf nodes:
  - CDATA sections:
    - `""` (tag name) = `![CDATA[`
    - `@text` = everything before the closing `]]>`
  - All other special declarations (comments, processing instructions, DOCTYPE, entities, etc.):
    - `""` (tag name) = type identifier (e.g., `?xml`, `!DOCTYPE`, `!--`, `!ENTITY`, `?xml-stylesheet`)
    - `@text` = everything after the opening, until (excluding) the closing `>` or `?>`
    - Examples:
      - `<!-- comment -->`: `"" = "!--"`, `@text = " comment "`
      - `<?xml version="1.0"?>`: `"" = "?xml"`, `@text = " version=\"1.0\""`
      - `<!DOCTYPE html>`: `"" = "!DOCTYPE"`, `@text = " html"`
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
  - CDATA sections preserved distinctly from regular text
  - Fully lossless round-tripping of XML/HTML documents
- Input only due to non-bijective nature with JSON
