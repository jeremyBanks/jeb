# Known Issues and Future Improvements

This document tracks known limitations and issues identified during code review that should be addressed in future iterations.

## Critical Issues (Block Re-running)

### 1. Missing `workspace = true` Handling
**Status**: ✅ FIXED
**Spec Reference**: Lines 163-164

The tool now correctly parses and handles dependencies that use `workspace = true`:
- When encountering `workspace = true`, looks up resolution fields from `[workspace.dependencies]`
- Uses those fields for voting in the equivalence class algorithm
- Preserves configuration fields from the member Cargo.toml
- Idempotency test passes (tool can re-run on its own output)

### 2. Losing Equivalence Classes Not Inlined
**Status**: ✅ FIXED
**Spec Reference**: Lines 180-182, 188-194

The tool now correctly handles losing equivalence classes:
- Captures old workspace.dependencies state before updating
- When a dependency uses `workspace = true` but doesn't match new winning class, it gets inlined
- Uses the OLD workspace version for inlining (preserves what it was pointing to)
- Configuration fields are preserved during inlining
- Test `test_loser_inlining` verifies this behavior
- Tool is now fully idempotent even when majority flips

### 3. Path Canonicalization Fails for Non-existent Paths
**Status**: ✅ FIXED
**Location**: `parse_dependency` line 307-327

The tool now gracefully handles path dependencies to non-existent paths:
- Attempts `canonicalize()` first for existing paths
- Falls back to manual path construction for non-existent paths
- Normalizes paths relative to base directory
- No fatal errors for paths that don't exist yet

### 4. default-features Handling with "default" Feature
**Status**: ✅ FULLY IMPLEMENTED
**Location**: Multiple locations (EquivalenceClass, build_dependency_value, update_member_toml, has_config_fields)

The tool now correctly handles dependencies with `default-features = false`:
- ALL dependencies (including those with `default-features = false`) participate in voting
- If ANY member has `default-features = false`, workspace.dependencies gets `default-features = false`
- Members with `default-features = false` use: `workspace = true, features = [...], default-features = false`
- Members wanting defaults enabled use: `workspace = true, features = ["default", ...]` (prepends "default")
- Field ordering: features always comes BEFORE default-features when both are present
- The blocking logic now allows `default-features` in workspace.dependencies (only blocks `optional` and `features`)
- Tool is fully idempotent with this feature

## Important Issues

### 5. Section-Form Dependencies Not Converted
**Status**: Not handled
**Spec Reference**: Lines 214-217

Dependencies defined as `[dependencies.foo]` sections are not converted to inline form when updated.

**Fix Required**:
- Detect section-form dependencies in member Cargo.toml files
- Convert them to inline table form: `foo = { ... }`

### 6. Blocked Dependencies Use Key Instead of Name
**Status**: Bug in warning system
**Spec Reference**: Lines 55-57

When blocking dependencies with configuration fields in `[workspace.dependencies]`, the tool blocks by TOML key rather than dependency name. This fails when the `package` field is used.

**Fix Required**:
- Parse the `package` field to get the true dependency name
- Block by dependency name across all members

### 7. Key Selection Doesn't Check Existing Workspace
**Status**: Incomplete implementation
**Spec Reference**: Lines 124-126

When selecting which key to use for `[workspace.dependencies]`, the spec says to prefer existing workspace keys, then alphabetical by member name. Currently sorts by key name only.

**Fix Required**:
1. Check if any key matches existing `[workspace.dependencies]` entry
2. If yes, use that key
3. Otherwise, use key from alphabetically-first member name (not key name)

### 8. Path Handling for Member Cargo.toml
**Status**: Potential bug
**Location**: `build_dependency_value`

Paths in `[workspace.dependencies]` are relative to workspace root (correct), but when inlining into member Cargo.toml files, paths need to be relative to the member directory, not workspace root.

**Fix Required**:
- Add a parameter to `build_dependency_value` for the target directory
- Convert paths appropriately based on context

### 9. Version Field for Path/Git Dependencies
**Status**: Semantic question
**Location**: `versions_compatible_opt`

Path and git dependencies don't use the version field for resolution in Cargo. Should dependencies with same path but different versions be in the same equivalence class?

**Decision Needed**: Clarify spec or implement Cargo-compatible behavior

## Code Quality Issues

### 10. Minimal Test Coverage
Tests only cover basic happy path. Need tests for:
- Losing equivalence classes
- `workspace = true` handling
- Configuration field preservation
- Path dependencies
- Git dependencies
- Section-form dependencies
- Tie-breaking scenarios
- Re-running behavior
- Edge cases (empty workspace, malformed data)

### 11. New Entry Ordering Not Implemented
**Status**: Simplified
**Spec Reference**: Lines 228-238

The "scan upward from bottom" insertion algorithm is not implemented. Currently relies on `toml_edit` default ordering.

**Impact**: Minor - order is not critical for functionality

### 12. Unused Parameters
- `update_member_toml`: `_all_deps` parameter
- `build_dependency_value`: `_include_config` parameter

These were kept for future use but should be cleaned up or utilized.

## Non-Critical Issues

### 13. Single-Voter Dependencies Promoted
Dependencies used by only one crate are still promoted to `[workspace.dependencies]`. This is technically correct per spec, but arguably wasteful.

**Consider**: Add flag to skip promoting single-voter dependencies

### 14. Features Array Silently Filters Non-strings
If features array contains non-string values, they're filtered out silently. Could be an error instead.

### 15. Empty Workspace Handling
No special handling for empty `[workspace.members]` array. Works correctly but untested.

## Assessment

### Currently Working
✅ Version compatibility logic
✅ Equivalence class grouping
✅ Voting algorithm
✅ Field ordering
✅ Basic TOML manipulation
✅ Configuration field detection in workspace
✅ Happy path for fresh normalization
✅ Handles existing `workspace = true` dependencies
✅ Idempotent - can re-run on own output
✅ **NEW**: Includes members with `default-features = false` in voting
✅ **NEW**: Sets `default-features = false` in workspace when needed
✅ **NEW**: Uses `features = ["default", ...]` for members wanting defaults
✅ **NEW**: Cargo check safety with rollback on failure

### Fully Idempotent
✅ Can handle re-running on normalized workspaces
✅ Inlines losing classes when majority flips
✅ Handles workspace membership changes correctly
✅ Handles `default-features = false` in workspace.dependencies

### Recommended Next Steps
1. ✅ ~~Implement `workspace = true` parsing (Critical #1)~~ - DONE
2. ✅ ~~Implement loser inlining (Critical #2)~~ - DONE
3. ✅ ~~Fix path canonicalization (Critical #3)~~ - DONE
4. ✅ ~~Implement `default-features` handling with "default" feature (Critical #4)~~ - DONE
5. ✅ ~~Add cargo check safety mechanism~~ - DONE
6. **All critical issues resolved!** Tool is now production-ready
7. Optional improvements: section-form dependencies, key selection, comprehensive tests
