# Jeb CLI Design Document

## Overview

`jeb` is a command-line tool for processing data through a pipeline of transformations. It operates on a heterogeneous stream of items, where each item can be either text (JSON-like) or binary (Bencode-like) data.

The core philosophy is Unix-like composability with explicit state management - commands read from and write to a shared vector of items, enabling flexible data transformation workflows.

## Data Model

### Type Structure

The pipeline state is `Vec<Item>`, where:

```rust
Vec<Item>

enum Item {
    Text(Vec<JsonValue>),     // JSON-like data model
    Binary(Vec<BencodeValue>)  // Bencode-like data model
}
```

**Key characteristics:**
- The top-level `Vec<Item>` is heterogeneous - can contain both Text and Binary items
- Each `Item` contains a homogeneous `Vec` of values in that format
- Commands have defined behavior for each mode (Text vs Binary)
- Explicit commands exist to convert between Text and Binary modes

### Text Mode (JSON-like)

Based on JSON data model with extensions:

- **Strings**: UTF-8 text strings
- **Numbers**: i64, u64, or f64
- **Booleans**: true/false
- **Null**: null value
- **Arrays**: ordered sequences
- **Objects**: key-value maps with **preserved key order** (important!)

### Binary Mode (Bencode-like)

Similar to Bencode but without sorted keys requirement:

- **Byte Strings**: raw binary data (not UTF-8)
- **Numbers**: i64 only
- **Lists**: ordered sequences
- **Dictionaries**: key-value maps with **unsorted keys**
- **No booleans or nulls**

### I/O Modes

- **stdin**: Binary mode (reads raw bytes)
- **stdout**: Binary mode (writes raw bytes)
- **File I/O**: Can be either, depending on content/command

### Common Usage Patterns

In simple cases, items are just strings:
- `Text(vec!["hello".into()])` - single UTF-8 string
- `Binary(vec![b"data".into()])` - single byte string

But the model supports rich structured data:
- `Text(vec![object, array, number])` - multiple JSON values
- `Binary(vec![dict, list, int])` - multiple Bencode values

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

Commands have defined behavior for each mode:

**Example: `encode-jeb85`**
- On `Text` items: operates on UTF-8 strings within the JSON values
- On `Binary` items: operates on byte strings within the Bencode values

**Example: `split-lines`**
- On `Text` items: splits UTF-8 strings on `\n`
- On `Binary` items: splits byte strings on `\n` byte

**Example: Arithmetic operations** (future)
- On `Text` items: can operate on i64/u64/f64 numbers
- On `Binary` items: can only operate on i64 numbers

### Mode Conversion Commands

Explicit commands to convert between modes:

**`to-text`** / **`as-text`** - Convert Binary items to Text items
- Attempts UTF-8 decoding of byte strings
- Converts Bencode values to JSON equivalents where possible
- May error on invalid UTF-8

**`to-binary`** / **`as-binary`** - Convert Text items to Binary items
- Encodes UTF-8 strings as byte strings
- Converts JSON values to Bencode equivalents
- Booleans/nulls may need special handling or error

**`parse-json`** - Parse text/binary strings as JSON, creating Text items

**`parse-bencode`** - Parse binary strings as Bencode, creating Binary items

**`serialize-json`** - Serialize Text items to JSON string representation

**`serialize-bencode`** - Serialize Binary items to Bencode byte string representation

### Source/Sink Behavior

**Sources:**
- `stdin` - reads raw bytes, creates `Binary(vec![bytes])`
- File reads - creates `Binary(vec![bytes])` by default
- `help` - creates `Text(vec![string])`

**Sinks:**
- `stdout` - expects Binary items, writes raw bytes
- If given Text items, may need implicit conversion or error
- `write:path` - writes Binary items as raw bytes

### Migration Path

Current `Vec<Bytes>` → Future `Vec<Item>`:
1. Treat existing `Bytes` as `Binary(vec![bytes])`
2. Gradually add Text support to commands
3. Add conversion commands
4. Update sources/sinks to be mode-aware

## Design Goals

### 1. Intelligent Auto-Completion

**Sources**: If no source commands are present, prepend `stdin`
- Sources: `stdin`, `help`, `self`, file paths, future network sources, etc.

**Sinks**: If no sink commands are present, append `stdout`
- Sinks: `stdout`, future file output, network sinks, etc.

This allows:
```bash
jeb encode-jeb85                    # reads stdin, writes stdout
jeb self encode-jeb85               # reads self, writes stdout
jeb help                            # outputs help to stdout
jeb stdin split-lines ./output.txt  # reads stdin, outputs to file
```

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

Then prepend/append as needed before execution.

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

1. **Implicit conversions**: Should `stdout` automatically convert Text to Binary (via UTF-8 encoding)? Or require explicit `to-binary`?

2. **Mixed-mode operations**: How do commands behave when state contains both Text and Binary items? Apply to each according to type? Error? Filter?

3. **Nested values**: How do commands like `split-lines` work on `Text(vec![object, array])`? Do they only apply to string values? Recursively search for strings?

4. **Boolean/null handling**: When converting Text to Binary, how to handle booleans and nulls? Error? Convert to string representation? Skip?

5. **Number precision**: When converting Binary (i64 only) to Text (i64/u64/f64), how to choose the type?

6. **Bencode dict keys**: Bencode traditionally requires byte string keys. Do we enforce this or allow other types?

7. **Empty items**: Can an Item contain an empty Vec? `Text(vec![])` or `Binary(vec![])`? What does this represent?

### Workflow Questions

8. **Mode vs Filter unification**: Should `--last` be different from `last`? Current thinking: yes, keep separate.

9. **Mode persistence**: Sticky vs next-only? Current thinking: sticky (Option A).

10. **Command discoverability**: How do users know which commands respect mode overrides? Needs documentation/help system.

11. **Merge semantics**: How exactly does merge work with the new data model? Merging within Items or across Items?

12. **Chain necessity**: Is `chain` just a no-op? Or does it have meaning in certain contexts?

13. **Error propagation**: How do commands signal errors in a pipeline? Current: `Result<Vec<Bytes>, Panic>`. Future: `Result<Vec<Item>, Panic>`?

14. **Streaming vs buffering**: Should some operations stream through items rather than buffering entire state?

15. **Multiple inputs/outputs**: Do we ever need commands that take N inputs and produce M outputs explicitly?

## Implementation Phases

### Phase 0: Data Model Migration
- Define `Item` enum with Text and Binary variants
- Implement JsonValue and BencodeValue types with preserved/unsorted key order
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
