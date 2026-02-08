# External File Storage for Inline Snapshots

## Goals

Allow inline snapshots to automatically store large values in external files
instead of embedding them in source code, with automatic switching based on size
thresholds.

**Target behavior:**

- Values >= 128 lines → store in external file
- Values <= 96 lines → store inline in source
- Hysteresis (96/128) prevents flapping when values hover near threshold

## Constraints

1. **Type signature must be preserved**:
   `snapshot<T>(value: T) -> InlineCell<T>`
   - Cannot return a wrapper type like `InlineCell<External>`
   - The value type `T` must be the actual data type

2. **All `Value` types must continue to work inline**
   - External storage is an optimization, not a replacement
   - Types that don't support external storage stay inline regardless of size

3. **Crate aliasing**: When emitting code that references our types, we can use
   `::inline::TypeName` because Rust 2018+ resolves `::crate_name` by **package
   name**, not dependency alias

4. **Parsing limitation**: We can serialize any `Bake` type to Rust code, but we
   cannot parse Rust code back into values at runtime (no `Unbake` trait, would
   need a Rust interpreter)

## Solution: Side-Channel Communication

### The Problem with Wrapper Types

A wrapper type approach like `snapshot(External("path"))` fails because:

- `T` becomes `External`, not the actual value type
- Return type would be `InlineCell<External>` instead of `InlineCell<String>`

### The Solution: Thread-Local Stash

Use a side-channel to communicate the external path without polluting the type
signature:

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

1. `external("path.snap")` stashes the path keyed by thread ID, returns
   `String::new()`
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

1. **Registry keying**: Should external snapshots use the same registry as
   inline, or separate? Same seems fine - the key is (file, index, type),
   storage location is an implementation detail.

2. **Verify mode behavior**: When external file doesn't exist in verify mode,
   should we:
   - Panic (strict)
   - Fall back to inline value (lenient)
   - Create the file (auto-fix)

   Recommendation: Panic with helpful error message.

3. **Git integration**: Should `.snapshots/` be gitignored or committed?
   - Committed: snapshots are part of test fixtures
   - Ignored: regenerate on each run

   Recommendation: Committed (like any snapshot test framework).

4. **Naming collision**: What if two files have same name in different
   directories?
   - `src/foo.rs` → `src/.snapshots/foo_0.snap`
   - `tests/foo.rs` → `tests/.snapshots/foo_0.snap`

   No collision - `.snapshots` is per-directory.

---

## Alternative: Macro-Based `outside!` Approach

The above design uses a function-based side-channel approach. This section
explores an alternative: a macro that uses `include_bytes!`/`include_str!` to
compile external file contents into the binary, maintaining the "no runtime file
dependency" property for verify mode.

### Motivation

The function-based `external()` approach has a limitation: reading external
files at runtime means verify mode depends on those files existing. With a macro
using `include_*!`, the file contents are embedded at compile time, so verify
mode needs no file I/O at all.

### Core Idea

```rust
// Usage:
outside!("snapshots/large_output.txt").value = generate_large_output();

// The data is compiled in via include_str!/include_bytes!
// Write mode updates the external file
// Verify mode compares against compiled-in data
```

### Type Support via `TryFrom<&[u8]>`

To support both `String` and `Vec<u8>` (and potentially other types), use a
trait bound:

```rust
pub trait OutsideValue: Sized + PartialEq {
    /// Parse from raw bytes (file contents)
    fn from_bytes(bytes: &[u8]) -> Result<Self, OutsideError>;

    /// Serialize to raw bytes (for file writing)
    fn to_bytes(&self) -> Vec<u8>;
}

impl OutsideValue for String {
    fn from_bytes(bytes: &[u8]) -> Result<Self, OutsideError> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| OutsideError::Utf8(e))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl OutsideValue for Vec<u8> {
    fn from_bytes(bytes: &[u8]) -> Result<Self, OutsideError> {
        Ok(bytes.to_vec())
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }
}
```

The return type is inferred from usage context:

```rust
// Type inferred as OutsideCell<String> from .value assignment
outside!("output.txt").value = some_string;

// Type inferred as OutsideCell<Vec<u8>>
outside!("output.bin").value = some_bytes;
```

### Macro Expansion (Declarative Macro Approach)

A declarative macro using `include_bytes!` (works for both String and Vec<u8>):

```rust
#[macro_export]
macro_rules! outside {
    ($path:literal) => {{
        // Embed file contents at compile time
        static __OUTSIDE_DATA: &'static [u8] = include_bytes!($path);

        // Compute path for runtime file operations
        // file!() gives source file, $path is relative to it
        $crate::OutsideCell::new(
            __OUTSIDE_DATA,
            file!(),      // Source file path
            $path,        // Relative path to snapshot
            line!(),
            column!(),
        )
    }};
}
```

**Path resolution at runtime:**

```rust
impl<T: OutsideValue> OutsideCell<T> {
    pub fn new(
        compiled_data: &'static [u8],
        source_file: &'static str,
        relative_path: &'static str,
        line: u32,
        column: u32,
    ) -> Self {
        // Resolve absolute path: source_file's directory + relative_path
        let source_dir = Path::new(source_file).parent().unwrap_or(Path::new("."));
        let absolute_path = source_dir.join(relative_path);

        // Parse compiled data into T
        let value = T::from_bytes(compiled_data)
            .expect("compiled snapshot data should be valid");

        OutsideCell {
            compiled_data,
            absolute_path,
            value,
            dirty: false,
            line,
            column,
        }
    }
}
```

### The Chicken-and-Egg Problem

**Problem:** `include_bytes!("file.txt")` fails at compile time if `file.txt`
doesn't exist. But you need to run the code to create the file.

**Solutions:**

#### Option A: Explicit "New" Flag (APPROVED for MVP)

Use an explicit flag or prefix in the macro to indicate the file doesn't exist
yet:

```rust
// File exists - include it
outside!("snapshots/output.txt").value = generate_output();

// File doesn't exist yet - explicit "new" marker
outside!(new "snapshots/output.txt").value = generate_output();
// Or alternative syntax:
outside_new!("snapshots/output.txt").value = generate_output();
```

The `new` variant skips `include_bytes!` and uses empty data. First write-mode
run creates the file, then user removes the `new` flag and recompiles.

**Advantages:**

- Explicit intent - clear when bootstrapping vs established
- No magic file-existence detection at compile time
- Simple declarative macro implementation
- User controls when to "promote" to normal mode

```rust
#[macro_export]
macro_rules! outside {
    (new $path:literal) => {{
        // Bootstrap mode: no file yet, use empty data
        static __OUTSIDE_DATA: &'static [u8] = &[];
        $crate::OutsideCell::new(
            __OUTSIDE_DATA,
            file!(),
            $path,
            line!(),
            column!(),
        )
    }};
    ($path:literal) => {{
        // Normal mode: file must exist
        static __OUTSIDE_DATA: &'static [u8] = include_bytes!($path);
        $crate::OutsideCell::new(
            __OUTSIDE_DATA,
            file!(),
            $path,
            line!(),
            column!(),
        )
    }};
}
```

#### Option B: Manual File Creation

Create an empty/placeholder file before first compilation:

```bash
mkdir -p snapshots
touch snapshots/output.txt
cargo test  # Now compiles
```

#### ~~Option C: Proc Macro with Fallback~~ (REJECTED)

A proc macro could theoretically check file existence at compile time and emit
empty data if missing. However, this approach is **rejected** because:

- Affects build caching in unpredictable ways (file existence checks aren't
  tracked by cargo)
- `Span::source_file()` is unstable (requires nightly)
- Adds proc-macro crate dependency and complexity
- The explicit `new` flag (Option A) is cleaner and more predictable

#### Option D: Separate Init Tool (NOT PLANNED)

A `cargo inline init` command that:

1. Parses source files for `outside!("...")` calls
2. Creates missing snapshot files with empty content
3. Run before first compilation

### OutsideCell Structure

```rust
pub struct OutsideCell<T: OutsideValue> {
    /// Compiled-in data (from include_bytes!)
    compiled_data: &'static [u8],

    /// Absolute path to external file (for write mode)
    absolute_path: PathBuf,

    /// Current value (parsed from compiled_data)
    value: T,

    /// Has the value been modified?
    dirty: bool,

    /// Source location (for error messages)
    line: u32,
    column: u32,
}

/// Inner type for DerefMut pattern (matches InlineCell's approach)
pub struct OutsideInner<T: OutsideValue> {
    pub value: T,
    cell: *mut OutsideCell<T>,  // Back-reference for write
}
```

### Mode Behavior

Uses the same `INLINE_MODE` environment variable as inline cells:

| Mode                          | Behavior                                                                              |
| ----------------------------- | ------------------------------------------------------------------------------------- |
| **Verify** (default in tests) | Compare `.value` assignment against compiled-in data. Panic on mismatch. No file I/O. |
| **Write**                     | Write new value to external file. Next recompilation picks up changes.                |
| **Memory**                    | Accept any value, no persistence.                                                     |
| **Reject**                    | Panic on any `.value` assignment.                                                     |

```rust
impl<T: OutsideValue> Drop for OutsideCell<T> {
    fn drop(&mut self) {
        if !self.dirty {
            return;
        }

        match get_mode() {
            Mode::Verify => {
                // Compare against compiled-in data
                let expected = T::from_bytes(self.compiled_data)
                    .expect("compiled data should be valid");
                if self.value != expected {
                    panic!(
                        "Outside snapshot mismatch at line {}!\n\
                         Expected (compiled): {:?}\n\
                         Got: {:?}\n\
                         File: {}",
                        self.line,
                        expected,
                        self.value,
                        self.absolute_path.display()
                    );
                }
            }
            Mode::Write => {
                // Write to external file
                let bytes = self.value.to_bytes();
                if let Some(parent) = self.absolute_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&self.absolute_path, &bytes)
                    .expect("failed to write outside snapshot");
            }
            Mode::Memory => {
                // Nothing to do
            }
            Mode::Reject => {
                panic!("Outside snapshot modification rejected (INLINE_MODE=reject)");
            }
        }
    }
}
```

### Comparison: `outside!` vs `snapshot()` + `external()`

| Aspect                         | `snapshot(external(...))` | `outside!(...)`                                |
| ------------------------------ | ------------------------- | ---------------------------------------------- |
| **Compile-time embedding**     | No                        | Yes (include_bytes!)                           |
| **Runtime file read (verify)** | Yes                       | No                                             |
| **Requires file at compile**   | No                        | Yes (or proc macro fallback)                   |
| **Implementation complexity**  | Lower (no macro)          | Higher (macro + proc-macro for bootstrap)      |
| **Type support**               | String only (initially)   | Any OutsideValue (String, Vec<u8>, extensible) |
| **Source modification**        | Yes (rewrites call)       | No (file path is static)                       |

### Key Differences from Inline Snapshots

| Aspect            | Inline (`snapshot()`)            | Outside (`outside!()`)           |
| ----------------- | -------------------------------- | -------------------------------- |
| **Data location** | Embedded in source code          | External file (compiled in)      |
| **Serialization** | `databake::Bake` → Rust tokens   | Raw bytes (UTF-8 for String)     |
| **Write target**  | Source file (modifies Rust code) | External file (no source change) |
| **Registry**      | Yes (file, index, type)          | No (path is the key)             |
| **Diffing**       | AST comparison                   | Byte comparison                  |
| **git diff**      | Shows in source file             | Shows in snapshot file           |

### Integration with Existing Code

The `outside!` macro would be a parallel system to `snapshot()`:

**Shared:**

- `INLINE_MODE` environment variable and `Mode` enum
- `get_mode()` function
- Conceptual model (verify vs write)

**Separate:**

- No registry (path is implicit key)
- Different cell type (`OutsideCell<T>` vs `InlineCell<T>`)
- Different serialization (`to_bytes()` vs `Bake`)
- No source code modification

**File structure:**

```
crates/inline/
├── src/
│   ├── lib.rs              # Add: pub mod outside; pub use outside::*;
│   ├── outside.rs          # NEW: OutsideCell, OutsideValue trait, outside! macro
│   ├── inline.rs           # Unchanged
│   └── runtime.rs          # Unchanged (Mode is shared)
└── tests/
    └── outside_test.rs     # NEW: tests for outside!
```

### ~~Potential Proc-Macro Crate~~ (NOT PLANNED)

A proc-macro crate was considered but rejected due to build caching complexity.
The declarative macro with explicit `new` flag is the approved approach.

### Open Questions for `outside!`

1. **Proc macro or declarative?** → DECIDED: Declarative with explicit `new`
   flag

2. **Line ending normalization?**
   - Should `\r\n` vs `\n` differences cause verification failure?
   - Recommendation: normalize to `\n` on both read and write

3. **Trailing newline?**
   - Should files always end with newline?
   - Recommendation: preserve exactly what's written (no auto-add)

4. **Binary vs text distinction?**
   - Could have `outside!` for text (String) and `outside_bytes!` for binary
   - Or infer from type annotation / return type context
   - Recommendation: single macro, type inferred from context

5. **Snapshot directory convention?**
   - Allow arbitrary paths: `outside!("../fixtures/data.txt")`
   - Or enforce convention: `outside!("foo")` → `snapshots/foo.snap`
   - Recommendation: allow arbitrary paths for flexibility

### Implementation Phases (APPROVED)

**Phase 1: Declarative Macro with Explicit `new` Flag (MVP)**

1. Implement `OutsideValue` trait for String and Vec<u8>
2. Implement `OutsideCell<T>` with mode-aware Drop
3. Declarative `outside!` macro with two variants:
   - `outside!("path")` - includes file via `include_bytes!`
   - `outside!(new "path")` - bootstrap mode, empty data
4. Document workflow: start with `new`, run write mode, remove `new`

**Phase 2: Extended Type Support (FUTURE)**

1. More `OutsideValue` implementations
2. Potentially serde-based generic impl for any Serialize+Deserialize
