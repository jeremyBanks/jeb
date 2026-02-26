# jeb Component Inventory

*Generated 2026-02-26. Living document tracking what exists.*

## Status: CLI Works ✓

```bash
echo "Hello World" | cargo run -p jeb --bin jeb -- stdin to-hex stdout
# Output: 48656c6c6f20576f726c640a
```

---

## Current CLI Commands

### Sources
| Command | Status | Description |
|---------|--------|-------------|
| `stdin` | ✓ Works | Read all stdin into state |
| `./path` | ✓ Works | Read file at path |
| `self` | ✓ Works | Read own executable |

### Sinks
| Command | Status | Description |
|---------|--------|-------------|
| `stdout` | ✓ Works | Write all state to stdout |

### Encoding/Decoding
| Command | Status | Description |
|---------|--------|-------------|
| `to-hex` | ✓ Works | Bytes → hex string |
| `parse-hex` | ✓ Works | Hex string → bytes |
| `to-binary` | ✓ Works | Bytes → binary string (0/1) |
| `parse-binary` | ✓ Works | Binary string → bytes |
| `to-base64` | ✓ Works | Bytes → base64 string |
| `parse-base64` | ✓ Works | Base64 string → bytes (whitespace-tolerant) |
| `encode-z85` | ✓ Works | Z85 encode (via z855 crate) |
| `decode-z85` | ✓ Works | Z85 decode |
| `encode-jeb85` | ✓ Works | Same as encode-z85 currently |

### JSON
| Command | Status | Description |
|---------|--------|-------------|
| `parse-json` | ✓ Works | Parse JSON text into Value |
| `to-json` | ✓ Works | Serialize Value to compact JSON |
| `to-json-pretty` | ✓ Works | Serialize Value to pretty JSON |

### Splitting/Chunking
| Command | Status | Description |
|---------|--------|-------------|
| `split-lines` | ✓ Works | Split by newline |
| `split-shell` | ✓ Works | Shell-style tokenization |
| `split-whitespace` | ✓ Works | Split by whitespace |
| `split-N` | ✓ Works | Split into N-byte chunks (supports K/M/G suffixes) |

### Joining
| Command | Status | Description |
|---------|--------|-------------|
| `join` | ✓ Works | Concatenate all items |
| `join-lines` | ✓ Works | Join with newlines |
| `join-space` | ✓ Works | Join with spaces |

### Selection
| Command | Status | Description |
|---------|--------|-------------|
| `first` | ✓ Works | Keep only first item |
| `last` | ✓ Works | Keep only last item |
| `first-N` | ✓ Works | Keep first N items |
| `last-N` | ✓ Works | Keep last N items |

### Filtering
| Command | Status | Description |
|---------|--------|-------------|
| `filter` | ✓ Works | Remove empty items |
| `collapse` | ✓ Works | Merge adjacent same-type items |
| `find-X` | ✓ Works | Keep items containing X |

### Aliases
| Alias | Expands To |
|-------|------------|
| `to-jeb85-lines` | `encode-jeb85 split-80 join-lines` |

---

## Missing from Conceptual Model

Based on `history-pit/_.md/20251129-fa3339-69404869-slop-CONCEPTUAL-MODEL.md`:

### Not Yet Implemented
- `parse-xml` / `to-xml` — XML support
- `sort` — Sort items
- `chain` — Concatenate multiple streams
- `merge` — Interleave streams by ordering
- `map` / `flat_map` — Transform functions
- `join-array` / `split-array` — Array aggregation
- `by-lines` / `by-null` — Chunking that preserves delimiters
- Error pipeline infrastructure
- DAG-based execution model (currently linear only)

### Data Model Gap
Current CLI uses `Vec<Bytes>` for state, not the full `jeb_value::Value` model.
The conceptual model describes three types: Text, Bytes, Structured.
jeb-stream has `Item` with these three variants, but CLI doesn't fully use it.

---

## Crate Structure

### jeb-value (`crates/jeb-value/`)
**Status: Comprehensive**

Core value types:
- `Null`, `Boolean`, `Number` — primitives
- `Bytes`, `String` — binary and text
- `Array` — sequences
- `BytesMap`, `StringMap` — maps with binary/text keys

Features:
- Full `Ord` implementation with canonical ordering
- Serde integration (serialize/deserialize)
- serde_json interop (with feature flag)
- Extensive tests

Spec: `TRACEY.md` (38KB of detailed requirements)

### jeb-stream (`crates/jeb-stream/`)
**Status: Solid Foundation**

`Item` enum:
- `Bytes(Bytes)` — binary data
- `Text(Arc<str>)` — text data  
- `Value(Value)` — structured data

Sources: `stdin`, `read_path`, `text_source`, `bytes_source`
Sinks: `stdout`, `write_path`
Transforms: `lines`, `split_after`, `chunks`, `to_hex`, `parse_hex`, `to_binary`, `parse_binary`, `split_whitespace`, `collapse`, `filter`, `split_shell`

TODO items from `TODO.md`:
- `as_value`, `as_bytes`, `as_text` — type coercion
- `box` / `unbox` — value wrapping
- `flat_map`, `map` — transforms
- `fail_fast` — error handling
- `serde(f)` — serde-based conversion
- `command(Command)` — subprocess integration

### jeb-common (`crates/jeb-common/`)
**Status: Utilities**

- `shell_tokenizer` — shell-style argument parsing
- `bi/` — bijective integer encodings (hilbert, z_order, spiral, etc.)
- `text/b1032` — B1032 encoding (decimal + hex-letter system)
- `Panic` type — error wrapper
- Testing utilities

### jeb (main crate, `crates/jeb/`)
**Status: CLI + Library Shell**

- Binary: `src/bin/jeb.rs` — command-line tool
- Library: re-exports jeb-common, jeb-stream, jeb-value
- Model types in `src/model/`

---

## Design Documents

| File | Content |
|------|---------|
| `history-pit/.../slop-CONCEPTUAL-MODEL.md` | Full vision: Text/Bytes/Structured, DAG execution, error pipelines |
| `history-pit/.../spec-data-model.md` | Data model requirements |
| `crates/jeb-value/TRACEY.md` | Detailed jeb-value specification |
| `crates/jeb-stream/TODO.md` | Missing stream transforms |
| `crates/jeb/examples/DESIGN-CONSTRAINTS.md` | Z85 extension analysis (35KB) |

---

## Test Status

```bash
cd /Users/matte/jeb
cargo test -p jeb-value    # Value types + serde
cargo test -p jeb-stream   # Stream transforms
cargo test -p jeb-common   # Common utilities
cargo test -p jeb          # Main crate
```

---

## Next Steps (Suggestions)

1. **JSON support** — Add `parse-json` and `to-json` to CLI, using jeb-value
2. **Base64** — Common encoding, easy to add
3. **Use Item properly** — CLI should work with `Item` not just `Bytes`
4. **Sort** — Leverage `Value::Ord` for canonical sorting
5. **Tests** — Verify all CLI commands have test coverage
