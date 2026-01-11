# Save Crate - Complete Feature Analysis

This document provides a comprehensive overview of all features, options, and ideas in the `save` crate - both implemented and planned.

## Overview

`save` is a command-line tool designed as a "no questions asked" Git commit utility. The core philosophy is to commit everything in the current directory automatically with intelligent defaults.

**Version**: 0.20220708.0 (July 2022)

```
╔══════════════════╗╔════╗
║Would you like to ║║►YES║
║SAVE the changes? ║║ NO ║
╚══════════════════╝╚════╝
```

---

## 1. Core Features (Fully Implemented)

### 1.1 Auto-Generated Commit Messages

The tool generates structured commit messages that encode repository metadata:

**Format**: `[r|s|z]N [/ gG] [/ nC] [/ xHHHH] [/ oHHHH]`

| Component | Description |
|-----------|-------------|
| `r/s/z` | Prefix indicating repository state (regular/shallow/z-mode) |
| `N` | Revision index (commits in first-parent chain) |
| `gG` | Generation index (max topological distance from roots) - only if different from N |
| `nC` | Commit index (total reachable commits - 1) - only if different from G |
| `xHHHH` | First 4 hex digits of tree hash (uppercase) |
| `oHHHH` | Origin (last 4 hex of root commit ID) - omitted for root commits |

**Example Messages**:
- `r0 / xABCD` - First commit (root)
- `r123 / xDEF1` - 124th commit, linear history
- `r50 / g75 / n100 / x2345 / o1234` - Commit with merge history
- `s9 / g15 / n71 / xABCD` - Shallow clone with merges
- `z2 / x1234 / o5678` - Z-mode (hit depth limit)

### 1.2 Hash Prefix Brute-Forcing

The tool brute-forces commit timestamps to produce commits with specific hash prefixes.

**Default behavior**: The first 4 hex digits of the commit hash will match the tree hash (e.g., commit `c795...` for tree `C795...`).

**Special prefix characters**:
- `_` - Wildcard (any nibble value)
- `C` - Next nibble of minimum-timestamped commit ID
- `R` - Last digits of revision index
- `G` - Last digits of generation index
- `N` - Last digits of commit index

### 1.3 Z-Mode (Depth-Limited Scanning)

For large repositories, the tool uses bounded-complexity graph walking:

- **Default max_depth**: 255 commits
- **Trust hierarchy**: `r` commits (non-shallow), `s` commits (shallow repos only), `z` commits (only in z-mode)
- **Z-mode triggers**: When ANY parent path hits the depth limit
- **Performance**: O(1) best case (trusting parent), O(max_depth) worst case

### 1.4 Shallow Clone Support

Full support for shallow Git clones:
- Detects shallow repositories via `.git/shallow`
- Uses `s` prefix for shallow commits
- Treats shallow boundary commits as effective roots
- Calculates origin from boundary commits

---

## 2. CLI Options (Documented)

### 2.1 Content Options

| Option | Short | Description | Status |
|--------|-------|-------------|--------|
| `--all` | `-a` | Commit all files (default) | Implemented |
| `--staged` | `-s` | Only staged files (like `git commit`) | Implemented |
| `--tree <TREE>` | | Use specific tree object directly | Implemented |
| `--empty` | `-e` | Empty commit (same tree as parent) | Implemented |
| `--allow-empty` | | Allow commit with no changes | Implemented |

### 2.2 Commit Options

| Option | Short | Description | Status |
|--------|-------|-------------|--------|
| `--message <MSG>` | `-m` | Custom commit message | Implemented |
| `--message-prefix <PREFIX>` | `-M` | Prefix before auto-message | Implemented |
| `--prefix <HEX>` | `-x` | Target commit hash prefix | Implemented |
| `--head <HEAD>` | | Branch to update | Partial (type mismatch) |
| `--no-head` | `-n` | Don't update any refs (dry run) | Implemented |

### 2.3 Signature Options

| Option | Short | Description | Status |
|--------|-------|-------------|--------|
| `--timestamp <TS>` | `-t` | Override timestamp | Implemented |
| `--timeless` | `-0` | Deterministic timestamps | Documented |
| `--author <AUTHOR>` | | Override author | Implemented |
| `--committer <COMMITTER>` | | Override committer | Documented |

### 2.4 History Options

| Option | Short | Description | Status |
|--------|-------|-------------|--------|
| `--max-depth <N>` | | Graph walk depth limit | Implemented (default: 255) |
| `--rebuild` | | Ignore messages, full graph walk | Implemented |
| `--add-parent <REF>` | `-p` | Add additional parent | Documented |
| `--remove-parent <REF>` | | Remove a parent | Documented |
| `--squash` / `--amend` | `-u` | Squash into parent | Documented (TODO) |
| `--squash-to <REF>` | | Squash to ancestor | Documented (TODO) |
| `--squash-after <REF>` | | Squash after branch point | Documented (TODO) |
| `--squash-all` | | Squash entire repo | Documented (TODO) |
| `--retcon-to-ref <REF>` | | Rewrite history to ancestor | Documented (TODO) |
| `--retcon-after <REF>` | | Rewrite after branch point | Documented (TODO) |
| `--retcon-all` | | Rewrite entire history | Documented (TODO) |

### 2.5 Environment Variables

All major options support environment variables:
- `SAVE_COMMIT_MESSAGE`
- `SAVE_COMMIT_PREFIX`
- `SAVE_ALLOW_EMPTY`
- `SAVE_TIMESTAMP`
- `SAVE_TIMELESS`
- `SAVE_AUTHOR`
- `SAVE_COMMITTER`
- `SAVE_HEAD`
- `SAVE_NO_HEAD`
- `SAVE_MAX_DEPTH`
- `SAVE_REBUILD`
- `SAVE_ADD_PARENT`
- `SAVE_REMOVE_PARENTS`
- `SAVE_SQUASH_COUNT`
- `SAVE_SQUASH_TO`
- `SAVE_SQUASH_AFTER`
- `SAVE_SQUASH_ALL`
- `SAVE_RETCON_TO`
- `SAVE_RETCON_AFTER`
- `SAVE_RETCON_ALL`
- `RUST_LOG` (verbosity)

---

## 3. Library/API Features

### 3.1 Easy Functions (`ez.rs`)

```rust
// Commit all changes
save::all()

// Commit specific paths (TODO - falls back to all)
save::paths(&["file1.rs", "file2.rs"])

// Custom configuration
save::with(|o| {
    o.message = Some("Custom message".into());
    o.allow_empty = true;
})
```

### 3.2 Extension Traits

**`RepositoryExt`** for `git2::Repository`:
- `working_index()` - Get index with working tree contents
- `temporary()` - Create repo in temp directory
- `signature_or_fallback()` - Get signature with fallbacks
- `save()` - Commit working directory

**`CommitExt`** for `git2::Commit`:
- `to_bytes()` - Raw commit object bytes
- `graph_stats()` - Calculate graph statistics
- `squashed(depth)` - Squash with ancestors (TODO)
- `brute_force_timestamps()` - Find hash-prefix-matching timestamps

**`OidExt`** for `git2::Oid`:
- `from_array()` - Fast Oid construction
- `for_object()` - Calculate object hash

---

## 4. Utility Modules

### 4.1 ZigZag Encoding (`zigzag.rs`)

Implements standard signed/unsigned integer encoding:
- `ZigZag` trait: Convert between signed and unsigned integers
- Used for compact storage of signed values

### 4.2 ZugZug Pairing (`zigzag.rs`)

Implements 2D Cantor pairing function:
- `ZugZug` trait: Map pairs of integers to/from single integers
- Used for brute-forcing timestamp pairs (author/committer)

### 4.3 Hex Parsing (`hex.rs`)

Hex string parsing with mask support:
- `decode_hex_nibbles()` - Parse hex with wildcards
- `MaskedBytes` - Bytes with validation mask
- `_` wildcard for "don't care" nibbles
- `hex!` macro for compile-time parsing

### 4.4 Graph Statistics (`graph_stats.rs`)

Abstract graph walking algorithm:
- `CommitView` trait - Abstract commit interface
- `RepositoryView` trait - Abstract repository interface
- `MessageParser` - Parse save-format messages
- `GraphStatsCalculator` - Main calculation engine

---

## 5. Unimplemented/Incomplete Features

### 5.1 Squashing (Documented, Not Implemented)

The squash functionality is documented but marked as TODO:
- `--squash` / `--amend` - Squash into parent
- `--squash-to <REF>` - Squash up to ancestor
- `--squash-after <REF>` - Squash branch commits
- `--squash-all` - Squash entire repo

The `squashed()` method in `git2.rs:490-509` has skeleton code but returns `todo!()`.

### 5.2 Retcon (History Rewriting)

History rewriting features are documented but not implemented:
- `--retcon-to-ref` - Rewrite timestamps/authorship to ancestor
- `--retcon-after` - Rewrite after branch point
- `--retcon-all` - Rewrite entire history

### 5.3 Selective Path Committing (`ez.rs`)

The `save::paths()` function is documented but falls back to committing all:
```rust
// TODO: Implement selective path committing
Save::with(|o| o.all = true).save()
```

### 5.4 Timeless Mode

The `--timeless` / `-0` flag is documented but its implementation may be incomplete. Intent: produce deterministic timestamps for reproducible builds by using timestamps relative to parent commits.

### 5.5 `signature_or_fallback()`

The method in `git2.rs:95-163` ends with `todo!()`.

---

## 6. Future Topics (From Documentation)

### 6.1 Rollback Handling

Open questions:
- How to handle reverting commits made with save?
- Do we need special logic to detect reverts?
- Should reverted commits affect next commit's metadata?

### 6.2 History Normalization

Open questions:
- When should users run `--rebuild`?
- Should we detect "wrong" history and suggest rebuild?
- Can we incrementally fix history?
- How to handle mixed save/manual commits?

---

## 7. Technical Details

### 7.1 Dependencies

Key dependencies:
- `git2` - Git operations
- `clap` - CLI parsing with derive
- `petgraph` - Graph data structures
- `sha-1` - Hash calculations
- `tracing` - Logging
- `parking_lot` - Thread synchronization
- `num_cpus` - Multi-threaded brute-forcing

### 7.2 Performance

- Brute-forcing uses all CPU cores
- Graph walking bounded by `max_depth` (default 255)
- O(1) optimization when parent has trusted message
- Hash prefix matching uses XOR and mask comparison

### 7.3 Testing

- 14 unit tests for graph statistics
- Snapshot tests for usage help
- Integration tests for hex/zigzag encoding
- Self-hosting (dogfooding) - tool commits using itself

---

## 8. Summary: Implementation Status

| Category | Implemented | Documented Only | TODO |
|----------|-------------|-----------------|------|
| Core commit | All content options | - | - |
| Message generation | Full format | - | - |
| Hash brute-forcing | Working | - | - |
| Z-mode | Complete | - | - |
| Shallow support | Complete | - | - |
| Squashing | - | CLI flags | Core logic |
| Retcon | - | CLI flags | Everything |
| Selective paths | - | `paths()` API | Implementation |
| Timeless mode | Partial | Full | Verification |
| Parent manipulation | - | Add/remove | Everything |

---

## 9. Recommended Next Steps

1. **Complete squashing**: The most useful missing feature
2. **Implement selective paths**: Simple addition to the ez API
3. **Verify timeless mode**: Ensure reproducible timestamp behavior
4. **Decide on retcon**: Whether to implement or remove
5. **Consider rollback**: Define behavior for reverts
6. **Add `--head` branch support**: Fix the type mismatch (currently `i64` instead of `String`)
