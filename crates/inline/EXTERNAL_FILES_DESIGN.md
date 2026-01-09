# External File Storage for Inline Snapshots

## Goals

Allow inline snapshots to automatically store large values in external files instead of embedding them in source code, with automatic switching based on size thresholds.

**Target behavior:**
- Values >= 128 lines → store in external file
- Values <= 96 lines → store inline in source
- Hysteresis (96/128) prevents flapping when values hover near threshold

## Constraints

1. **Type signature must be preserved**: `snapshot<T>(value: T) -> InlineCell<T>`
   - Cannot return a wrapper type like `InlineCell<External>`
   - The value type `T` must be the actual data type

2. **All `Value` types must continue to work inline**
   - External storage is an optimization, not a replacement
   - Types that don't support external storage stay inline regardless of size

3. **Crate aliasing**: When emitting code that references our types, we can use `::inline::TypeName` because Rust 2018+ resolves `::crate_name` by **package name**, not dependency alias

4. **Parsing limitation**: We can serialize any `Bake` type to Rust code, but we cannot parse Rust code back into values at runtime (no `Unbake` trait, would need a Rust interpreter)

## Solution: Side-Channel Communication

### The Problem with Wrapper Types

A wrapper type approach like `snapshot(External("path"))` fails because:
- `T` becomes `External`, not the actual value type
- Return type would be `InlineCell<External>` instead of `InlineCell<String>`

### The Solution: Thread-Local Stash

Use a side-channel to communicate the external path without polluting the type signature:

```rust
use std::{
    collections::HashMap,
    sync::Mutex,
    thread::ThreadId,
};
use once_cell::sync::Lazy;

static PENDING_EXTERNAL: Lazy<Mutex<HashMap<ThreadId, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[track_caller]
pub fn external(path: &str) -> String {
    let tid = std::thread::current().id();
    PENDING_EXTERNAL.lock().unwrap().insert(tid, path.to_string());
    String::new()
}
```

**How it works:**

1. `external("path.snap")` stashes the path keyed by thread ID, returns `String::new()`
2. `snapshot(...)` checks for a stashed path for the current thread
3. If found: external mode (read/write to file)
4. If not found: inline mode (current behavior)
5. Stash entry is consumed (removed) immediately

**Why this works:**
- Rust evaluates function arguments before the function call
- `external()` runs first, stashes path, returns empty String
- `snapshot()` runs immediately after, finds and consumes the stash
- Thread-keying prevents races in parallel tests

### Usage

```rust
// Explicit external file
snapshot(external("data/big_output.snap")).value = generate_large_output();

// Inline (current behavior, unchanged)
snapshot("small value".to_string()).value = actual;
```

## Type Support

### Initial Implementation: String Only

External files only support `String` initially:
- Trivial serialization: write string bytes to file
- Trivial deserialization: read file as string
- Covers the main use case (large text output)

Other `Value` types continue to work but stay inline regardless of size.

### Future Expansion

Can expand to more types later via sealed trait:

```rust
mod sealed {
    pub trait ExternalStorage: Sized {
        fn read_from(path: &std::path::Path) -> std::io::Result<Self>;
        fn write_to(&self, path: &std::path::Path) -> std::io::Result<()>;
    }
}

impl sealed::ExternalStorage for String { /* ... */ }
impl sealed::ExternalStorage for Vec<u8> { /* ... */ }
// Future: serde-based implementations, or even Rust code evaluation
```

## Automatic Mode Switching

### Detection Logic

In `snapshot()`, after consuming any pending external path:

```rust
let is_external = if let Some(path) = pending_external_path {
    // Explicit external() wrapper was used
    true
} else if external_file_exists_for_this_location() {
    // Previously auto-switched to external
    true
} else {
    false
};
```

### Write Logic (on value change)

```rust
let baked = value.bake(&env);
let line_count = count_lines(&baked.to_string());

if is_currently_external {
    if line_count <= 96 {
        // Switch external → inline
        write_to_source(baked);
        delete_external_file();
        rewrite_source_to_remove_external_wrapper();
    } else {
        // Stay external
        write_to_external_file(&value);
    }
} else {
    if line_count >= 128 && T is String {
        // Switch inline → external (only for String)
        let ext_path = compute_external_path();
        write_to_external_file(&value);
        rewrite_source_to_add_external_wrapper(ext_path);
    } else {
        // Stay inline
        write_to_source(baked);
    }
}
```

### Source Rewriting for Auto-Switch

When auto-switching modes, we need to rewrite the source:

**Inline → External:**
```rust
// Before:
snapshot("...128+ lines of content...".to_string())

// After:
snapshot(external(".snapshots/file_42.snap"))
```

**External → Inline:**
```rust
// Before:
snapshot(external(".snapshots/file_42.snap"))

// After:
snapshot("...96 or fewer lines...".to_string())
```

This is AST manipulation - replace the argument tokens entirely.

### External File Path Computation

For auto-generated paths, use the stable index (not line/column which changes):

```rust
fn compute_external_path(source_file: &Path, stable_index: usize) -> PathBuf {
    let stem = source_file.file_stem().unwrap();
    source_file
        .parent()
        .unwrap()
        .join(".snapshots")
        .join(format!("{}_{}.snap", stem, stable_index))
}
```

Example: `tests/foo.rs` snapshot #3 → `tests/.snapshots/foo_3.snap`

## Implementation Plan

### Phase 1: Manual External Files

1. **Add `external()` function** (`src/external.rs`)
   - Thread-local stash for pending path
   - Returns `String::new()`
   - Uses `#[track_caller]`

2. **Modify `snapshot()` to check stash**
   - At start: check for pending external path, consume it
   - Store external path in `InlineCellInner` if present

3. **Add `ExternalCell` storage variant**
   - New field: `external_path: Option<PathBuf>`
   - On drop/write: if external, write to file instead of source
   - On verify: if external, read from file instead of source

4. **Path resolution**
   - External paths relative to source file directory
   - Use `resolve_source_path()` logic (already exists)

5. **Tests**
   - Manual external file usage
   - Thread safety with parallel tests
   - Verify mode with external files
   - Write mode with external files

### Phase 2: Automatic Switching

1. **Line counting**
   - After baking, count newlines in token stream string
   - Cache the count to avoid repeated computation

2. **Threshold detection**
   - Compare against 96 (→ inline) and 128 (→ external)
   - Only trigger for `String` type (check `TypeId`)

3. **Source rewriting for mode switch**
   - Inline → External: replace argument with `external("path")`
   - External → Inline: replace `external("path")` with baked value
   - Uses existing `runtime::update_macro_by_index` infrastructure

4. **External file lifecycle**
   - Create `.snapshots/` directory if needed
   - Delete external file when switching to inline
   - Handle orphaned files (file exists but source doesn't reference it)

5. **Tests**
   - Auto-switch at threshold
   - Hysteresis (no flapping near threshold)
   - Mode persistence across runs

### Phase 3: Future Enhancements (Not In Scope)

- `Vec<u8>` support via sealed trait
- Serde-based support for arbitrary types
- Configurable thresholds
- Rust code evaluation for external files (exotic)

## File Structure

```
crates/inline/
├── src/
│   ├── lib.rs              # Add: pub mod external; pub use external::external;
│   ├── external.rs         # NEW: external() function, stash, ExternalStorage trait
│   ├── inline.rs           # Modify: check stash, external_path field
│   ├── registry.rs         # Modify: handle external storage in registry key?
│   └── runtime.rs          # Modify: external file read/write helpers
└── tests/
    └── external_serial_test.rs  # NEW: external file tests
```

## Open Questions

1. **Registry keying**: Should external snapshots use the same registry as inline, or separate? Same seems fine - the key is (file, index, type), storage location is an implementation detail.

2. **Verify mode behavior**: When external file doesn't exist in verify mode, should we:
   - Panic (strict)
   - Fall back to inline value (lenient)
   - Create the file (auto-fix)

   Recommendation: Panic with helpful error message.

3. **Git integration**: Should `.snapshots/` be gitignored or committed?
   - Committed: snapshots are part of test fixtures
   - Ignored: regenerate on each run

   Recommendation: Committed (like any snapshot test framework).

4. **Naming collision**: What if two files have same name in different directories?
   - `src/foo.rs` → `src/.snapshots/foo_0.snap`
   - `tests/foo.rs` → `tests/.snapshots/foo_0.snap`

   No collision - `.snapshots` is per-directory.
