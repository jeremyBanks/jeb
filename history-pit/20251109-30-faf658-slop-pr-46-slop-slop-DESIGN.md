# jeb Library Design

## Overview

jeb is a collection of composable utilities for working with async streams of
JSON objects. The library uses futures streams with serde_json and IndexMap for
JSON representation (subject to change). The CLI is a convenient interface for
composing library operations.

## Design Principles

1. **Streaming with minimal memory usage**: All operations work on streams to
   avoid loading entire datasets into memory
2. **Graceful degradation**: Handle unexpected input reasonably rather than
   failing
3. **Composability**: Each utility is a building block that can be combined with
   others

## Core Components

### 1. Input Parsing

Converts text streams into streams of parsed JSON objects.

**Behavior:**

- Accepts JSON lines (newline-delimited), JSON arrays, concatenated objects, and
  mixed formats with arbitrary text
- Simple scan for `{` character, then delegates to real JSON parser to extract
  complete object
- Between top-level objects, any character (including `{` in text) is ignored

### 2. Output Serialization

Converts streams of JSON objects into text streams.

**Format (fixed, no configuration options):**

```
[{"first":"object"}
,{"next":"object"}
,{"another":"object"}
]
```

**Properties:**

- Valid JSON array when read as complete file
- Line-processable: strip first character of each line to get individual JSON
  objects
- Final `]` emitted when stream closes

### 3. JSON Total Ordering

Defines a total ordering for JSON values used by other utilities.

**Type precedence:**

- Types are ordered by the ASCII/lexicographic ordering of their representative
  characters:
  - `"` for strings (lowest)
  - `0` for numbers
  - `[` for arrays
  - `f` for false
  - `n` for null
  - `t` for true
  - `{` for objects (highest)

**Within-type ordering:**

- **Strings**: Lexicographic UTF-8 byte order
- **Numbers**: Numeric comparison
- **Arrays**: Element-by-element comparison; shorter arrays sort before longer
  arrays when all compared elements are equal
- **Objects**: Compared as flattened array `[key1, value1, key2, value2, ...]`,
  so key order matters

### 4. K-way Stream Merge

Merges multiple sorted input streams into a single sorted output stream.

**Signature:** Accepts custom comparison function (PartialOrd)

**Behavior:**

- Maintains round-robin pointer that advances on every pull
- Always start with the round-robin head as the current minimum
- Rotate through the other stream heads
- If a stream head compares less than the current minimum, it becomes the new
  minimum
- Equal or incomparable is not less
- Emit the minimum

**Assumptions:** Input streams assumed to be pre-sorted for typical merge
behavior; unsorted inputs handled gracefully (may produce unusual pull patterns)

### 5. Sorting Buffer

Buffers N elements and emits them in sorted order.

**Signature:** Accepts custom comparison function (Ord - total ordering
required)

**Behavior:**

- Buffers up to N elements
- Once buffer contains N elements: on each new input, emit current minimum and
  add new element to buffer
- At stream end: drain remaining elements in sorted order

**Purpose:** Handle mostly-sorted streams where elements may be out of order by
up to N positions

### 6. Key Ordering

Reorders keys within JSON objects.

**Options struct (with Default trait):**

```rust
struct KeyOrderOptions {
    recursive: bool,      // default: true
    first: Vec<String>,   // default: []
    last: Vec<String>,    // default: []
    sort: bool,          // default: true
}
```

**Behavior:**

- Places keys in `first` at beginning of object in specified order
- Places keys in `last` at end of object in specified order
- Remaining keys: sorted lexicographically if `sort` is true, otherwise preserve
  original order
- If `recursive` is true, applies to nested objects

### 7. Consecutive Group Reduction

Groups consecutive items and reduces each group to output items.

**Components:**

- **Grouping function:** Determines which consecutive items belong together
  (returns bool for equality)
- **Reduction functions:** Slice of `Fn(Vec<JsonValue>) -> Vec<JsonValue>`,
  applied sequentially
  - Functions take ownership of items (can modify in-place)

**Built-in reduction strategies:**

- **First**: Returns `vec![group[0]]`
- **Last**: Returns `vec![group[group.len()-1]]`
- **FirstAndLast**: Returns `vec![group[0], group[group.len()-1]]` (or just
  first if group size is 1)
- **Merge**: Merges objects' fields when there are no conflicts (a field exists
  in multiple objects with different values). Can merge nested objects
  recursively, but cannot merge conflicting primitive values (strings, numbers,
  arrays, booleans, null)

**Defaults:**

- Grouping: Total ordering equality
- Reduction: `[FirstAndLast]`

**Use case example:** Ten consecutive entities differ only in timestamp field.
With grouping function that ignores timestamp and reduction `[FirstAndLast]`,
emit first and last to capture timestamp range without redundant duplicates.

### 8. Re-rooting

Transforms a JSON tree by making a node at a specified path the new root.

**Signature:**
`fn reroot(json: JsonValue, path: &[&str]) -> Result<JsonValue, JsonValue>`

**Behavior:**

- Walks to node at specified path
- Makes that node the new root
- Nests original ancestors under the reversed path
- Returns `Err(original_json)` if:
  - Path does not exist
  - Key collision occurs during inversion (duplicate key would be created)

**Properties:**

- Reversible: re-rooting result at reversed path restores original structure
  (when no conflicts)
- Use case: Normalize representations where entities are embedded in containers,
  preserving container metadata

**Conceptual model:** JSON as directed graph where each property is an edge
labeled with the key name. Re-rooting flips edges along the path to create a new
root.

## Implementation Notes

- All stream transformations work with async streams (futures)
- Custom comparison functions enable flexibility while built-in total ordering
  provides sensible defaults
- Error handling philosophy: return original input on failure (same type
  signature) for easy fallback
