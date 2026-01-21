# Plan: Auto-manage Feature Dependencies from Cargo.lock

## Overview

Enhance `cargo_toml_normalize.rs` to automatically manage feature dependencies
based on the resolved dependency graph from `Cargo.lock`.

## Goals

1. **Auto-add transitive feature deps**: If optional dep A depends on optional
   dep B (per Cargo.lock), the feature for A should include B as a feature dep
2. **Sort internal deps first**: Feature deps matching this crate's optional dep
   names sort before external feature refs
3. **Smart cleanup**: Only delete auto-generated features when their backing dep
   is removed, with cascading removal of references

## Detailed Requirements

### 1. Auto-add Feature Dependencies

Given:
```toml
[dependencies]
serde = { optional = true }
serde_json = { optional = true }  # depends on serde per Cargo.lock
```

Currently generates:
```toml
[features]
serde = ["dep:serde"]
serde_json = ["dep:serde_json"]
```

Should generate:
```toml
[features]
serde = ["dep:serde"]
serde_json = ["serde", "dep:serde_json"]
```

**Algorithm**:
1. Parse workspace `Cargo.lock` to get dependency graph
2. For each optional dep in crate's Cargo.toml:
   - Look up its dependencies in the lockfile graph
   - Filter to only those that are ALSO optional deps in this crate
   - Add those as feature deps (the feature name, not `dep:name`)

### 2. Sorting Within Features

Current sort key: `(category, name)` where category is:
- 0: bare feature names
- 1: slash refs with /default
- 2: other slash refs
- 3: dep: refs

Proposed sort key: `(category, is_external, name)` where:
- For bare feature names (category 0):
  - `is_external = 0` if name matches an optional dep in this crate
  - `is_external = 1` otherwise

Example result:
```toml
serde_json = ["serde", "some_external_feature", "dep:serde_json"]
#             ^internal  ^external               ^dep ref
```

### 3. Smart Feature Cleanup

**Current behavior** (problematic):
- Removes invalid `dep:` refs pointing to non-existent deps
- Deletes features that become empty after cleanup
- This incorrectly deletes intentionally empty features like `_implicit_all = []`

**Proposed behavior**:
- Track which features are "auto-generated" (have `dep:X` where X matches feature name)
- Only delete a feature if:
  1. It had `dep:same_name` (was auto-generated for an optional dep)
  2. That dep no longer exists
  3. After removing invalid refs, feature would be empty OR only contain refs to
     other features being deleted in this pass
- Process deletions in dependency order (if feature A refs feature B, process B first)
- When deleting a feature, also remove all references to it from other features

**Example cascade**:

Before (deps `serde` and `serde_json` removed):
```toml
[features]
serde = ["dep:serde"]
serde_json = ["serde", "dep:serde_json"]
other = ["serde_json", "something_else"]
```

After:
```toml
[features]
other = ["something_else"]
```

Steps:
1. `dep:serde` invalid → `serde` feature marked for deletion
2. `dep:serde_json` invalid → `serde_json` marked for deletion
3. `serde_json` refs `serde` (being deleted) → ref removed, still has `dep:serde_json`
4. `dep:serde_json` removed → `serde_json` now empty, still deleted
5. `other` refs `serde_json` (deleted) → ref removed, keeps `something_else`

### 4. Implementation Steps

1. **Move/share lockfile parsing** from `workspace_deps.rs`:
   - Either move to a shared module
   - Or call from `cargo_toml_normalize.rs` via a shared interface

2. **Modify `normalize_crate_features()`**:
   - Accept dependency graph as parameter
   - After creating features for optional deps, add transitive deps
   - Pass optional dep names to sort function

3. **Modify `feature_dep_sort_key()`**:
   - Accept set of internal feature names
   - Return `(category, is_internal, name)` tuple

4. **Rewrite `clean_invalid_feature_deps()`**:
   - Identify auto-generated features (have `dep:same_name`)
   - Build deletion set based on missing deps
   - Iterate to handle cascading (feature A deleted → refs to A removed →
     feature B now empty and was auto-generated → B deleted)
   - Remove refs to deleted features from remaining features

5. **Update tests** to cover:
   - Transitive feature dep addition
   - Sorting with internal deps first
   - Cascade deletion
   - Preservation of intentionally empty features

## Files to Modify

- `crates/_autofix/src/cargo_toml_normalize.rs` - main logic
- `crates/_autofix/src/workspace_deps.rs` - share lockfile parsing
- Possibly create `crates/_autofix/src/lockfile.rs` for shared code

## Edge Cases

- Name normalization: `serde-json` (Cargo.toml) vs `serde_json` (feature name)
- Multiple versions of same package in lockfile
- Circular feature refs (shouldn't happen but handle gracefully)
- Features with both `dep:name` and other content shouldn't be deleted even if
  dep removed (only remove the invalid ref)
