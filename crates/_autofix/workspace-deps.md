# Cargo workspace dependency normalization

A tool for tidying Cargo workspace dependency definitions by promoting shared
dependencies to `[workspace.dependencies]` and having members inherit them with
`workspace = true`.

## Overview

This tool scans a Cargo workspace, analyzes all dependency definitions across
members, and normalizes them so that:

- Dependencies used by multiple crates (with compatible resolution fields) are
  defined in `[workspace.dependencies]` and inherited by members
- Dependencies that are no longer inherited by any member are removed from
  `[workspace.dependencies]`
- The tool is idempotent and re-runnable (like `cargo fmt`)

## Scope

### What we process

- `[dependencies]`
- `[dev-dependencies]`
- `[build-dependencies]`

### What we do NOT process

- Target-specific dependencies (e.g., `[target.'cfg(windows)'.dependencies]`)
- Any other Cargo.toml sections

### Dependency fields

We consider these fields as **resolution fields** (define where/what the
dependency is):

- `package`
- `version`
- `path`
- `git`
- `branch`
- `tag`
- `rev`
- `registry`

These fields are promoted to `[workspace.dependencies]` when inherited.

We do NOT promote these **configuration fields** (left in member Cargo.toml
files):

- `workspace`
- `optional`
- `features`
- `default-features`

If any configuration fields are found in `[workspace.dependencies]`, the tool
emits a warning and does not process that dependency name anywhere in the
workspace.

### Field ordering

When writing dependency tables, fields are ordered as follows:

1. `workspace`
2. `package`
3. `version`
4. `path`
5. `git`
6. `branch`
7. `tag`
8. `rev`
9. `registry`
10. `optional`
11. `features`
12. `default-features`

## Version format requirements

We only process version specifications in these formats:

- `"1.2.3"` (bare version)
- `"^1.2.3"` (caret prefix)

Both formats are treated identically when determining compatibility. Any other
format (e.g., `">=1.0, <2"`, `"~1.2"`, `"=1.2.3"`) is skipped and left
unchanged.

When writing versions to `[workspace.dependencies]`, we use the bare format
without the `^` prefix.

## Version compatibility

Two versions are considered **compatible** if their left-most non-zero
major/minor/patch component is the same:

- `1.2.3` and `1.5.0` are compatible (same major: 1)
- `0.2.3` and `0.2.7` are compatible (same minor: 2, since major is 0)
- `0.0.3` and `0.0.3` are compatible (same patch: 3, since major and minor are
  0)
- `1.2.3` and `2.0.0` are NOT compatible
- `0.2.3` and `0.3.0` are NOT compatible

Pre-release versions (e.g., `1.0.0-alpha`) are treated as exact matches only -
they are only compatible with themselves. A crate using `workspace = true` votes
for the exact pre-release version in the workspace.

## Dependency naming and the package field

The **dependency name** used for grouping is determined as follows:

- If the `package` field is present, use its value
- Otherwise, use the TOML table key

Example: Both of these refer to the same dependency (`tokio`):
```toml
# Member A
[dependencies]
tokio = "1.0"

# Member B
[dependencies]
tokio-crate = { package = "tokio", version = "1.0" }
```

These would be grouped together and normalized to use the same key (the first
one encountered alphabetically by member name, or the existing workspace entry's
key if present).

## Path dependencies

Relative paths in `path` fields are normalized relative to the workspace root
Cargo.toml before comparison. Two path dependencies are considered equal if
their normalized absolute paths are the same.

## Equivalence classes

Dependencies are grouped into **equivalence classes** based on:

1. The dependency name (as determined by the package field rules above)
2. All resolution fields must match exactly, EXCEPT for `version`
3. For `version`, the compatibility class (as defined above)

Two dependency definitions are in the same equivalence class if and only if:

- They have the same dependency name
- All non-version resolution fields are exactly equal
- Their versions are compatible (same compatibility class)

## Voting and selection

Each **crate** gets one vote per compatibility range for a given dependency. A
crate can vote for multiple compatibility ranges if it specifies incompatible
versions across different dependency sections (e.g., `foo = "1.2"` in
`[dependencies]` and `foo = "2.4"` in `[dev-dependencies]`), but it cannot have
multiple votes for the same compatibility range.

When a crate specifies the same dependency multiple times with compatible
versions (e.g., `foo = "1.2"` in `[dependencies]` and `foo = "1.5"` in
`[dev-dependencies]`), it contributes one vote for that compatibility range,
with the maximum version (`1.5`) as its suggested version. After normalization,
both sections will use `workspace = true`, pointing to version `1.5` (or higher
if other crates vote for higher compatible versions).

A crate using `workspace = true` for a dependency votes for the current
workspace version's equivalence class.

For each dependency name:

1. Group all definitions into equivalence classes
2. Count votes (one per crate per compatibility range) for each equivalence
   class
3. The equivalence class with the **most votes wins**
4. **Tie-breaker**: if two equivalence classes have the same vote count, compare
   using:
   - First, compare by maximum version using semver ordering
   - If versions are equal, sort all resolution fields by key name, then compare
     the resulting `[(key, value)]` lists lexicographically
5. Within the winning class, use the **maximum version** (by semver ordering) as
   the workspace version
6. All crates in the winning class inherit via `workspace = true` in all
   sections where they specify that dependency
7. All crates in losing classes have their dependency **inlined** in their own
   Cargo.toml with configuration fields preserved

## Re-running behavior

When the tool is re-run:

- If a previously-minority equivalence class now has more votes (due to new
  crates being added), it becomes the new winner
- Crates that were using `workspace = true` for the old winner get that version
  **inlined** into their Cargo.toml (using the version that was in
  `[workspace.dependencies]`), with their configuration fields preserved
- The new winner's version is promoted to `[workspace.dependencies]`
- Crates in the new winning class switch to `workspace = true`

## Cleanup

After processing, any entry in `[workspace.dependencies]` that is no longer
inherited by any member is removed.

## Formatting rules

### General preservation principle

Comments, whitespace, and all other formatting that does not need to be changed
should be preserved as-is, except where explicitly stated otherwise in this
specification.

### Dependency table syntax in member Cargo.toml files

- If `version` is the only field, use the simple string form: `foo = "1.2.3"`
- If any other fields are present, use the inline table form:
  `foo = { version = "1.2.3", optional = true }`
- The `[dependencies.foo]` section syntax is preserved when reading, but when
  updating such entries, they are converted to the inline table form and
  inserted using the ordering rules below

### Dependency table syntax in workspace Cargo.toml

- Always use the simple string form when version is the only field:
  `foo = "1.2.3"`
- Always use the inline table form when other fields are present:
  `foo = { version = "1.2.3", path = "../foo" }`
- The `[workspace.dependencies.foo]` section syntax, if encountered, is
  normalized to inline table form

### Ordering of new entries

Existing entries preserve their position. When inserting a new entry into a
non-empty table:

1. Scan upward from the bottom of the table
2. Find the first entry whose name sorts before the new entry's name
   (lexicographically)
3. Insert the new entry after that position

This preserves sorting if the table was already sorted, and appends near the
bottom otherwise.

## Error handling

The tool fails with a fatal error in the following cases:

- Malformed TOML files that cannot be parsed
- Invalid version strings that cannot be parsed by the `semver` crate
- Missing workspace members (paths in `[workspace].members` that don't exist or
  don't contain Cargo.toml files)
- Any I/O errors when reading or writing files

## Implementation notes

### Libraries to use

- `toml_edit` - for parsing and modifying Cargo.toml while preserving formatting
- `semver` - for parsing and comparing versions (provides total ordering)
- `glob` - for resolving workspace member patterns

### Algorithm outline

1. Find the workspace root Cargo.toml
2. Parse `[workspace].members` and resolve globs to find all member Cargo.toml
   files (fatal error if any glob matches nothing or paths are missing)
3. Check `[workspace.dependencies]` for configuration fields; emit warnings and
   skip those dependency names entirely
4. For each member, parse all dependencies from `[dependencies]`,
   `[dev-dependencies]`, and `[build-dependencies]`
5. Skip any dependency with:
   - Non-standard version format
   - Target-specific definition
6. Normalize all path dependencies relative to workspace root
7. Build equivalence classes for each dependency name
8. For each dependency name, determine the winning equivalence class (using vote
   count, then version + field ordering as tie-breaker)
9. Update `[workspace.dependencies]` with winners
10. Update member Cargo.toml files:
    - Winners get `workspace = true` (keeping configuration fields)
    - Losers get version inlined (keeping configuration fields)
11. Remove unused entries from `[workspace.dependencies]`
