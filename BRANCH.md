# Jeb CLI Design Document

## Overview

`jeb` is a command-line tool for processing data through a pipeline of transformations. It operates on a heterogeneous stream of items, where each item can serialize as either JSON (text) or Extended Bencode (binary).

The core philosophy is Unix-like composability with explicit state management - commands read from and write to a shared vector of items, enabling flexible data transformation workflows. The same rich data model can be serialized in either text (JSON) or binary (Extended Bencode) format.

## Data Model

### Type Structure

The pipeline state is `Vec<Item>`, where:

```rust
Vec<Item>

enum Item {
    Text(Vec<Value>),    // Values serialized as JSON
    Binary(Vec<Value>)   // Values serialized as Extended Bencode
}
```

**Key characteristics:**
- The top-level `Vec<Item>` is heterogeneous - can contain both Text and Binary items
- Each `Item` contains a homogeneous `Vec` of values in that format
- **Text and Binary support the same data model** - difference is serialization format only
- Commands have defined behavior for each mode (Text vs Binary)
- Explicit commands exist to convert between Text and Binary modes

### Unified Data Model

Both Text and Binary modes support the same rich data model:

- **Strings**: UTF-8 text strings (or byte strings in Binary mode)
- **Numbers**: i64, u64, or finite f64 (no NaN or Infinity)
- **Booleans**: true/false
- **Null**: null value
- **Arrays/Lists**: ordered sequences
- **Objects/Dictionaries**: key-value maps
  - **Text mode**: preserved key order (important!)
  - **Binary mode**: unsorted keys

**Float restrictions:**
- Only finite floats allowed (no NaN, no Infinity) to match JSON semantics
- **Specification constraint**, not enforced by wrapper types in the Value implementation
- CLI commands won't provide ways to create NaN or Infinity values
- Parsers (JSON, Extended Bencode) reject NaN/Infinity on input
- Future: may add debug assertions to catch violations during development

### Text Mode (JSON Serialization)

Serializes values as JSON text with preserved key order in objects.

Standard JSON syntax with extension:
- Objects maintain insertion order (not alphabetical)

### Binary Mode (Extended Bencode Serialization)

Serializes values as Extended Bencode binary format.

**Traditional Bencode:**
- `i<number>e` - integers (e.g., `i42e`, `i-17e`)
- `<length>:<bytes>` - byte strings (e.g., `4:spam`, `11:hello world`)
- `l...e` - lists (e.g., `li1ei2ee` for `[1, 2]`)
- `d...e` - dictionaries (e.g., `d3:key5:valuee`)

**Extended Bencode additions for JSON-completeness:**
- `f<number>e` - finite floats (e.g., `f3.14e`, `f-2.5e`, `f1.0e`)
  - Only finite values allowed (no NaN or Infinity)
  - Uses standard decimal representation
- `n` - null (single character)
- `b1` - boolean true
- `b0` - boolean false

**Benefits:**
- **Backward compatible**: Traditional Bencode still works
- **JSON-complete**: Can represent any JSON value
- **Round-trip both formats**: JSON ↔ Extended Bencode ↔ JSON
- **Unsorted keys**: Unlike traditional Bencode, no key sorting requirement

### I/O Modes

- **stdin**: Binary mode (reads raw bytes)
- **stdout**: Binary mode (writes raw bytes)
- **File I/O**: Can be either, depending on content/command

### Common Usage Patterns

In simple cases, items are just strings:
- `Text(vec!["hello".into()])` - single UTF-8 string, will serialize as JSON
- `Binary(vec![b"data".into()])` - single byte string, will serialize as Extended Bencode

But the model supports rich structured data:
- `Text(vec![object, array, number])` - multiple values, JSON serialization
- `Binary(vec![dict, list, float])` - same values, Extended Bencode serialization

Since both modes support the same data model, the choice between Text and Binary is primarily about:
- **Text**: Human-readable JSON, UTF-8 strings, preserved key order
- **Binary**: Compact Extended Bencode, byte strings, unsorted keys, backward-compatible with traditional Bencode

## Current Implementation

### Pipeline Model (Current)

Commands are executed left-to-right, currently operating on `Vec<Bytes>`:

```rust
state: Vec<Bytes> -> command1 -> command2 -> ... -> commandN -> output
```

**Note**: The current implementation uses the simple `Vec<Bytes>` model. Migration to the `Vec<Item>` model is planned.

### Command Categories (As Implemented)

1. **Sources** (add data to state):
   - `help` - adds README content
   - `stdin` - reads from stdin
   - `self` - reads the executable itself
   - `.` or `/` prefixed paths - reads files

2. **Filters** (select subset):
   - `first` - keep only first item
   - `last` - keep only last item
   - `first-N` - keep first N items
   - `last-N` - keep last N items
   - `filter` - remove empty items
   - `find-TARGET` - keep items containing TARGET bytes

3. **Splitters** (one→many):
   - `split-lines` - split on newlines
   - `split-N` - split by size with unit parsing (e.g., `split-64`, `split-1KiB`, `split-2MB`)

4. **Joiners** (many→one):
   - `join` - concatenate all items
   - `join-lines` - join with newlines between
   - `join-space` - join with spaces between

5. **Transformers** (modify in place):
   - `collapse` - collapse all whitespace to single spaces
   - `encode-z85` - Z85 encode each item
   - `encode-jeb85` - jeb85 encode each item

6. **Sinks** (consume state):
   - `stdout` - write all items to stdout, clear state
   - Future: `write:path` - atomic file writes

### Current Auto-Append Behavior

- If no commands provided: prepend `help`
- If `stdout` not last: append `stdout`

These are highlighted in red in diagnostic output to show implicit additions.

### Partially Implemented: Mode Flags

Mode flags exist but are not currently used:
- `--all` - operate on all items (intended default)
- `--last` - operate on last item only
- `--first` - operate on first item only

The `_default_mode` variable is set but never read (line 43 in jeb.rs).

## New Data Model Implications

### Command Behavior with Text vs Binary

Commands have defined behavior for each mode. Since both modes support the same data model, the difference is mainly about serialization format and string handling:

**Example: `encode-jeb85`**
- On `Text` items: operates on UTF-8 strings within values, preserves JSON serialization
- On `Binary` items: operates on byte strings within values, preserves Extended Bencode serialization

**Example: `split-lines`**
- On `Text` items: splits UTF-8 strings on `\n`, keeps as Text (JSON)
- On `Binary` items: splits byte strings on `\n` byte, keeps as Binary (Extended Bencode)

**Example: Arithmetic operations** (future)
- Both modes support i64/u64/f64 numbers thanks to Extended Bencode
- Binary mode serializes floats as `f<number>e`

### Mode Conversion Commands

Since both modes support the same data model, conversion is straightforward:

**`to-text`** / **`as-text`** - Convert Binary items to Text items
- Changes serialization format from Extended Bencode to JSON
- Attempts UTF-8 decoding of byte strings (may error on invalid UTF-8)
- All other value types convert directly (floats, bools, nulls all supported)

**`to-binary`** / **`as-binary`** - Convert Text items to Binary items
- Changes serialization format from JSON to Extended Bencode
- Encodes UTF-8 strings as byte strings
- All value types convert directly thanks to Extended Bencode

**`parse-json`** - Parse strings as JSON, creating Text items
- Parses UTF-8 or byte strings containing JSON
- Outputs Text items with parsed values

**`parse-bencode`** - Parse strings as Extended Bencode, creating Binary items
- Parses byte strings containing Extended Bencode
- Outputs Binary items with parsed values
- Supports both traditional and extended Bencode

**`serialize-json`** - Serialize values to JSON strings
- Takes Text or Binary items
- Outputs Text items containing JSON string representations

**`serialize-bencode`** - Serialize values to Extended Bencode byte strings
- Takes Text or Binary items
- Outputs Binary items containing Extended Bencode byte string representations

### Implicit Conversions

Many commands will automatically convert between formats as needed, using heuristics to determine the most appropriate conversion:

**Conversion principles:**
- Commands that need Text items will implicitly convert Binary → Text when needed
- Commands that need Binary items will implicitly convert Text → Binary when needed
- Commands that need parsed structures will attempt to parse strings automatically
- Commands that produce text output will serialize structured data automatically

**Example implicit conversions:**
```bash
# split-lines expects string data
# If input is Binary(vec![bencode_dict]), implicitly serializes to string first
jeb stdin parse-bencode split-lines

# encode-jeb85 operates on strings
# If input contains structured data, implicitly serializes to strings
jeb ./data.json parse-json encode-jeb85

# stdout expects Binary items
# If input is Text items, implicitly converts via as-binary
jeb help stdout
```

**Heuristics for conversion** (detailed in implementation):
- UTF-8 validity checks
- Structure detection (is this a string or complex object?)
- Format inference from command context
- Graceful degradation when conversion is ambiguous

**Note**: While implicit conversions provide convenience, explicit conversion commands (`as-text`, `as-binary`, `serialize-*`, `parse-*`) give precise control when needed.

### Format Auto-Detection

**`sniff`** - Auto-detect format and parse accordingly
- Buffers first 64KB of each item to detect format
- Automatically invokes appropriate parser based on detected format:
  - `parse-json` - for JSON data (starts with `{`, `[`, `"`, numbers, `true`, `false`, `null`)
  - `parse-bencode` - for Bencode/Extended Bencode data (starts with `i`, `l`, `d`, digit, `f`, `n`, `b`)
  - `parse-xml` - for XML/HTML data (starts with `<`)
  - `parse-protobuf` - for Protocol Buffer wire format (heuristic binary detection)
  - `as-text` - for valid UTF-8 text that doesn't match structured formats
  - `as-binary` - for binary data that isn't a recognized format
- Useful for generic data pipelines where input format may vary
- Can be combined with type-specific operations that follow

**Detection heuristics:**
1. Check UTF-8 validity first
2. If UTF-8, check for structured text format markers:
   - JSON: `{`, `[`, `"`, digits, `true`, `false`, `null`
   - XML/HTML: `<` followed by tag name or declaration
3. If not UTF-8 or no text format detected, check binary format markers:
   - Extended Bencode: `i`, `l`, `d`, `f`, `n`, `b`, or digit (for byte string length)
   - Protocol Buffers: heuristic checks for valid wire format structure
4. Fall back to `as-text` (UTF-8) or `as-binary` (non-UTF-8)

**Example usage:**
```bash
# Auto-detect and pretty-print various formats
jeb ./unknown-file sniff serialize-json

# Process any structured data format
jeb stdin sniff extract-field:name stdout

# Convert any format to Extended Bencode
jeb ./data.* sniff serialize-bencode write:./output.bencode
```

### XML Support (Input Only)

**`parse-xml`** - Parse XML/HTML to JSON representation

Special naming scheme for lossless round-tripping:
- `""` (empty string): The node's tag name
- `"-"`: Parent tag name
- `"--"`: Grandparent tag name (and so on)
- `"-attribute-name"`: Parent's attribute values
- `"--attribute-name"`: Grandparent's attribute values

**Virtual attributes** (always in data model):
- `@text`: Text content as first child (empty string for self-closing, null for no text)
- `@tail`: Text following the node
- `@index`: Sibling index for distinguishing identical adjacent parents

**Attribute handling:**
- Boolean attributes (HTML `<input disabled>`): `"disabled": true`
- Attributes with values: preserved as strings
- Attribute order preserved via ordered maps

**Metadata preservation:**
- CDATA sections: `"" = "![CDATA["`, `@text = content`
- Comments: `"" = "!--"`, `@text = " comment "`
- Processing instructions: `"" = "?xml"`, `@text = " version=\"1.0\""`
- DOCTYPE: `"" = "!DOCTYPE"`, `@text = " html"`

**Benefits:**
- Fully lossless round-tripping
- No collision with valid XML names (can't start with `-` or `@`)
- Preserves ordering and structure completely
- Input only (non-bijective with JSON, but can serialize back to XML)

**Example:**
```xml
<book id="123">
  <title>Example</title>
  <!-- comment -->
</book>
```
Becomes:
```json
{
  "": "book",
  "id": "123",
  "@text": "\n  ",
  "-id": "123",
  "--": null,
  "children": [
    {
      "": "title",
      "-": "book",
      "-id": "123",
      "@text": "Example",
      "@tail": "\n  "
    },
    {
      "": "!--",
      "@text": " comment ",
      "@tail": "\n"
    }
  ]
}
```

### Protocol Buffer Wire Format

**`parse-protobuf`** - Heuristic Protocol Buffer wire format decoder

Since wire format lacks schema information, uses best-effort auto-detection at each nesting level:

**Heuristic checks** (in order):
1. Valid UTF-8? → decode as string
2. Valid JSON? → parse as JSON (nested structured data)
3. Valid proto wire format? → decode recursively
4. Valid bencode? → decode as bencode (nested structured data)
5. Otherwise → treat as binary blob (base64 representation)

**Encoding support:**
- Can encode JSON structures to proto wire format
- Requires explicit type hints for fields (varint, fixed32, length-delimited, etc.)

**Limitations:**
- Best-effort without schema
- May misidentify nested binary data
- Not suitable for complex proto schemas without hints

**Example usage:**
```bash
# Decode proto wire format file
jeb ./message.pb parse-protobuf serialize-json

# Encode JSON to proto wire format (with type hints)
jeb ./data.json parse-json encode-protobuf write:./message.pb
```

### Source/Sink Behavior

**Sources:**
- `stdin` - reads raw bytes, creates `Binary(vec![bytes])`
- File reads - creates `Binary(vec![bytes])` by default
- `help` - creates `Text(vec![string])`

**Sinks:**
- `stdout` - writes Binary items as raw bytes
- If given Text items, implicitly converts to Binary (via serialization) before writing
- `write:path` - writes Binary items as raw bytes (with atomic replacement semantics)

### Migration Path

Current `Vec<Bytes>` → Future `Vec<Item>`:
1. Treat existing `Bytes` as `Binary(vec![bytes])` (Extended Bencode byte strings)
2. Implement Extended Bencode parser/serializer
3. Migrate commands to work with unified Value type
4. Add conversion commands (`to-text`, `to-binary`, etc.)
5. Update sources/sinks to be mode-aware

## Design Goals

### 1. Intelligent Auto-Completion

**Sources**: If no source commands are present, prepend `stdin` and `sniff`
- Sources: `stdin`, `help`, `self`, file paths, future network sources, etc.
- Auto-detection via `sniff` ensures input is parsed into structured data

**Sinks**: If no sink commands are present, append `as-text` and `stdout`
- Sinks: `stdout`, `write:path`, future network sinks, etc.
- Text conversion via `as-text` ensures human-readable output

**Default pipeline** (no explicit sources or sinks):
```bash
jeb [command]
# Expands to: stdin sniff [command] as-text stdout
```

This allows:
```bash
# Read stdin, auto-detect format, encode, output as text
jeb encode-jeb85
# Equivalent to: stdin sniff encode-jeb85 as-text stdout

# Explicit source, auto-detect sink
jeb self encode-jeb85
# Equivalent to: self encode-jeb85 as-text stdout

# Help output (already text)
jeb help
# Equivalent to: help as-text stdout

# Explicit source and sink
jeb stdin split-lines write:./output.txt
# No auto-completion needed
```

**Benefits**:
- Sensible defaults: parse structured input, output readable text
- Works with any format via `sniff` auto-detection
- No boilerplate for common use cases
- Can override by explicitly specifying sources/sinks

### 2. Command Operation Modes

Commands have default behaviors for how they process the state:

**Per-Item Operations** (default: operate on each item independently):
- `encode-z85`, `encode-jeb85`
- `collapse`
- Future: `decode-*`, `compress-*`, `hash-*`, etc.

**Aggregate Operations** (default: operate on last/single item):
- `split-lines`, `split-N`
- These naturally consume one item and produce many

**Selection Operations** (operate on entire state):
- `first`, `last`, `first-N`, `last-N`
- `filter`, `find-*`
- These inherently work on the full state

**Reduction Operations** (default: operate on all items):
- `join`, `join-lines`, `join-space`
- These naturally consume all items and produce one

### 3. Mode Flags Override Defaults

Mode flags can modify the default behavior of *most* commands:

```bash
jeb stdin split-lines encode-jeb85 stdout
# Default: split-lines operates on last item, encode-jeb85 on each item

jeb stdin --last encode-jeb85 stdout
# Force: encode-jeb85 operates only on last item

jeb stdin split-lines --all encode-jeb85 stdout
# Force: encode-jeb85 operates on all items (already the default here)
```

**Mode Scope**: Mode flags affect subsequent commands until another mode flag is encountered.

**Exceptions**: Some commands cannot have their mode changed:
- Selection operations like `first`/`last` must operate on entire state
- Reduction operations like `join` must consume all items
- May need annotation/documentation to indicate which commands respect modes

### 4. Unified Filtering and Modes

Consider whether filtering and mode selection should be unified:
- `--last` is semantically similar to `last` but doesn't remove other items from state
- Could have mode flags that also filter: `--only-last` (equivalent to `last`)?
- Or keep them separate for clarity

## Planned Command Additions

### Stream Combiners

**`chain`** - Sequential concatenation:
```
[A, B, C] -> [A, B, C]  (conceptually: outputs A, then B, then C in sequence)
```
Simply maintains order - likely just an identity operation or explicit pass-through.

**`merge`** - Sorted merge:
```
[B, D, F] + [A, C, E] -> [A, B, C, D, E, F]
```
Merges multiple streams by always taking the lexicographically smallest head element.
Treats each item in the state as a separate stream to merge.

### Sorting Buffers

**`sort-N`** - Bounded sort buffer:
- Maintains a sliding window of N elements
- Outputs elements in sorted order with up to N positions of reordering
- Useful for "mostly sorted" data or memory-constrained sorting

**`sort-all`** - Unbounded sort buffer:
- Buffers all items and outputs them sorted
- Equivalent to `join sort-lines split-lines` for line-based data
- May need variants: `sort-all-lines`, `sort-all-bytes`, etc.

### File Output Sink

**`write:path`** - Atomic file write:
Writes all items in state to a file with atomic replacement semantics. This is critical for safe in-place transformations where the same file is both input and output.

**Algorithm**:
1. Generate temporary filename: `NAME.(hex_timestamp_ms).tmp`
2. Write all state items to the temporary file
3. After complete write, perform atomic replacement:
   - **If target file exists**:
     - Rename existing file to `NAME.(hex_timestamp_ms).bak`
     - Rename `.tmp` to target filename
     - Unlink `.bak` file
   - **If target file doesn't exist**:
     - Simply rename `.tmp` to target filename

**Rationale**:
- Always write to `.tmp` first, even if target doesn't exist (protects against crashes mid-write)
- The rename operations are atomic on POSIX filesystems
- Backup file created only if replacing existing file
- If any step fails, the original file remains intact
- Avoids TOCTOU (time-of-check-time-of-use) races

**Safe detection**: Use `std::fs::metadata()` to check if target exists before rename dance. If it returns `Err(NotFound)`, skip the backup step.

**Example usage**:
```bash
# Safe in-place transformation
jeb ./data.txt split-lines filter join-lines write:./data.txt

# Process and save to new file
jeb stdin encode-jeb85 write:./output.jeb85

# Multiple transformations
jeb ./input.bin split-1MiB encode-z85 write:./output.z85
```

## Implementation Considerations

### Mode Flag Semantics

**Option A: Sticky Modes** (recommended)
```bash
jeb cmd1 --last cmd2 cmd3 --all cmd4
#    ^^^^ default  ^^^^ last ^^^^ last  ^^^^ all
```
Mode persists until changed.

**Option B: Next-Command-Only**
```bash
jeb cmd1 --last cmd2 cmd3
#    ^^^^ default  ^^^^ last ^^^^ default
```
Mode only affects the immediately following command.

### Command Metadata

Commands may need metadata annotations:
```rust
struct CommandMeta {
    name: &'static str,
    default_mode: Mode,  // All, Last, First
    respects_mode_override: bool,
    category: Category,  // Source, Sink, Filter, Transform, etc.
}
```

### State Model Clarity

Current model: `Vec<Bytes>` is a simple sequential list.

For operations like `merge`, may need to clarify:
- Is each `Bytes` item treated as an atomic unit?
- Or do we need streaming within items?
- How do we represent multiple "channels" for merge operations?

Likely: each `Bytes` is atomic, `merge` treats the Vec as multiple streams.

### Auto-Completion Detection

Need logic to detect presence of sources/sinks:
```rust
fn has_source(commands: &[String]) -> bool {
    commands.iter().any(|cmd| {
        matches!(cmd.as_str(), "stdin" | "self" | "help")
            || cmd.starts_with(".")
            || cmd.starts_with("/")
    })
}

fn has_sink(commands: &[String]) -> bool {
    commands.iter().any(|cmd| {
        matches!(cmd.as_str(), "stdout")
            || cmd.starts_with("write:")
        // future: network sinks
    })
}
```

Then auto-complete as needed:
```rust
if !has_source(commands) {
    commands.insert(0, "stdin".to_string());
    commands.insert(1, "sniff".to_string());
}

if !has_sink(commands) {
    commands.push("as-text".to_string());
    commands.push("stdout".to_string());
}
```

## Example Workflows

### Basic transformations
```bash
# Encode this script
jeb self encode-jeb85

# Split and encode
jeb self split-1KiB encode-jeb85

# Read, filter, encode
jeb ./data.bin find-MAGIC encode-z85
```

### With explicit modes
```bash
# Encode only the last chunk after splitting
jeb self split-lines --last encode-jeb85

# Operate on each item individually (explicit, though it's default)
jeb self split-lines --all encode-jeb85
```

### Sorting workflows
```bash
# Sort lines in a file
jeb ./data.txt split-lines sort-all join-lines

# Merge multiple sorted files
jeb ./sorted1.txt ./sorted2.txt ./sorted3.txt merge

# Use bounded sort for large data
jeb ./huge.txt split-lines sort-1000000 join-lines
```

### File I/O workflows
```bash
# In-place file transformation (safe atomic replacement)
jeb ./data.txt split-lines filter join-lines write:./data.txt

# Read from one file, write to another
jeb ./input.bin encode-jeb85 write:./output.jeb85

# Process stdin, save to file
jeb encode-z85 split-80 join-lines write:./encoded.txt

# Multiple transformations with file output
jeb ./binary.dat split-1MiB --all encode-jeb85 join-lines write:./chunks.txt
```

### Complex pipelines
```bash
# Read self, split by size, encode each chunk, merge back, output
jeb self split-64KiB encode-jeb85 chain
```

## Open Questions

### Data Model Questions

1. **Implicit conversions**: ~~Should `stdout` automatically convert Text to Binary?~~ **RESOLVED**: Yes, `stdout` and other commands perform implicit conversions as needed. See "Implicit Conversions" section for details.

2. **Mixed-mode operations**: How do commands behave when state contains both Text and Binary items? Apply to each according to type? Error? Filter?

3. **Nested values**: How do commands like `split-lines` work on `Text(vec![object, array])`? Do they only apply to string values? Recursively search for strings?

4. **Empty items**: Can an Item contain an empty Vec? `Text(vec![])` or `Binary(vec![])`? What does this represent?

5. **Dict key types**: Should Extended Bencode dictionaries allow any value type as keys (like JSON objects require strings)? Or only byte strings (traditional Bencode)?

6. **Float representation**: ~~How should floats serialize in Extended Bencode?~~ **RESOLVED**: Standard decimal representation only (e.g., `f3.14e`, `f-2.5e`). Scientific notation may be allowed for parsing. NaN and Infinity are not allowed per specification (match JSON semantics). Not enforced by wrapper types - parsers reject on input, CLI doesn't provide ways to create them.

### Workflow Questions

7. **Mode vs Filter unification**: Should `--last` be different from `last`? Current thinking: yes, keep separate.

8. **Mode persistence**: Sticky vs next-only? Current thinking: sticky (Option A).

9. **Command discoverability**: How do users know which commands respect mode overrides? Needs documentation/help system.

10. **Merge semantics**: How exactly does merge work with the new data model? Merging within Items or across Items?

11. **Chain necessity**: Is `chain` just a no-op? Or does it have meaning in certain contexts?

12. **Error propagation**: How do commands signal errors in a pipeline? Current: `Result<Vec<Bytes>, Panic>`. Future: `Result<Vec<Item>, Panic>`?

13. **Streaming vs buffering**: Should some operations stream through items rather than buffering entire state?

14. **Multiple inputs/outputs**: Do we ever need commands that take N inputs and produce M outputs explicitly?

## Implementation Phases

### Phase 0: Data Model Migration
- Define `Item` enum with Text and Binary variants
- Implement unified `Value` type supporting full data model
- Implement JSON serializer/parser with preserved key order
- Implement Extended Bencode serializer/parser (`f<float>e`, `n`, `b0`, `b1`)
- Migrate existing commands to work with `Vec<Item>` (treating as Binary mode)
- Add basic conversion commands: `to-text`, `to-binary`
- Update error handling to `Result<Vec<Item>, Panic>`

### Phase 1: Fix Current Implementation & Mode Support
- Implement the mode flag functionality (currently unused)
- Fix auto-append logic for sources and sinks
- Document each command's default mode behavior
- Ensure all commands have defined behavior for Text vs Binary

### Phase 2: Add Core Commands
- Implement `chain` and `merge` (needs data model clarification)
- Implement `sort-N` and `sort-all`
- Add `parse-json`, `parse-bencode`, `serialize-json`, `serialize-bencode`
- Add command metadata system
- Add `write:path` atomic file sink

### Phase 3: Rich Data Operations
- Commands for manipulating JSON/Bencode structures (get, set, delete keys)
- Array/list operations (map, filter, reduce-like operations)
- Arithmetic operations on numbers
- String manipulation beyond simple encoding

### Phase 4: Refinement
- Comprehensive help system showing modes and type behavior
- Error messages that suggest corrections
- Performance optimization for large pipelines
- Handle edge cases (mixed-mode operations, nested values, etc.)

### Phase 5: Expansion
- More encoders/decoders
- Compression commands
- Cryptographic operations
- Network sources/sinks
- Query languages (jq-like for JSON, similar for Bencode)
