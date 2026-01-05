# Git-Zoom Edge Cases and Boundary Conditions Analysis

This document systematically analyzes potential edge cases and boundary
conditions in the git-zoom implementation based on code inspection and test
coverage analysis.

## 1. Empty Files and Directories

### Current Behavior

- **Empty directories**: Fully supported via `git mktree` mechanism. The tree
  manipulation in `tree.rs` handles empty trees correctly.
- **Empty subtrees**: Explicitly tested via `--allow-empty` flag in integration
  tests (see `test_empty_subtree_with_allow_empty`).
- **Empty files**: Work correctly as blobs in git.

### Potential Issues: NONE IDENTIFIED

- The `--allow-empty` flag allows zooming into non-existent paths, creating
  empty subtrees.
- Empty trees are created with `git::empty_tree()` which returns a valid empty
  tree hash.
- Zoom out with `--deny-empty` explicitly checks for empty subtrees via
  `git::ls_tree()`.

### Test Coverage

- ✅ `test_empty_subtree_with_allow_empty()` - basic empty zoom
- ✅ `test_zoom_without_allow_empty_fails()` - error on non-existent path
- ✅ `test_deny_empty()` - rejects empty subtree on zoom out

---

## 2. Symlinks

### Current Status: NOT HANDLED EXPLICITLY

The code does not handle symlinks as a special case. Git itself represents
symlinks as blobs with mode `120000`.

### Potential Issues

1. **Symlinks in paths**: If a path component is a symlink, git resolves it. The
   code uses `git rev-parse <commit>:<path>` which should follow symlinks
   correctly.

2. **Symlinks as subtree root**: When zooming into a path that is a symlink:
   - Git's `rev-parse` treats it as whatever it points to
   - If it points to a blob or directory, behavior depends on git's
     interpretation
   - **Risk**: Undefined behavior if symlink creates a cycle or points outside
     the repository

3. **Tree traversal with symlinks**: In `tree.rs`, the
   `replace_subtree_recursive` function uses `git::ls_tree()` which returns
   blobs and trees. Symlinks would appear as blobs with special mode.
   - **Risk**: If a path component in the middle is a symlink (mode 120000), the
     code treats it as a blob and replaces it with an empty tree. This might
     break the expected symlink structure.

### Example Problematic Scenario

```
Original tree:
  src/lib -> (symlink to src/other)
  src/other/file.txt

Zooming into src/lib:
- ls-tree returns (120000, blob, <hash>, "lib")
- replace_subtree_recursive treats it as a blob
- Replaces with empty tree, breaking the symlink
```

### No Test Coverage

- ❌ No tests for symlinks in paths
- ❌ No tests for symlinks as intermediate path components
- ❌ No tests for symlink cycles or external references

---

## 3. Very Long File Paths

### Current Status: POTENTIALLY PROBLEMATIC

1. **Path length limits**:
   - Most filesystems support paths up to 4096 bytes (or 260 characters on
     Windows)
   - Git supports longer path names in tree objects
   - The code splits paths by `/` and processes them iteratively

2. **String handling**:
   - Rust strings are UTF-8 and use heap allocation for longer strings
   - No explicit length checks in `zoom_in.rs:normalize_path()`
   - Path splitting in `tree.rs:replace_subtree()` uses `.split('/')` which
     works for any length

3. **Git command invocation**:
   - Git commands are passed as arguments to shell
   - System command-line length limits (usually ~131KB on Linux)
   - Could theoretically overflow with extremely long paths

### Potential Issues

1. **Working directory operations**: When filesystem paths exceed 4096 bytes,
   operations like `git reset --hard` might fail, but error handling exists.

2. **Path normalization**: The `normalize_path()` function in `zoom_in.rs`
   doesn't validate the final path length after joining, only checks for invalid
   components like `..`.

3. **Output parsing**: When `git ls-tree` returns paths with unusual encoding,
   `from_utf8_lossy()` is used, which could silently truncate or misinterpret
   data.

### Example Problematic Scenario

```rust
// Very deep nesting
let path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/... (very long)";
normalize_path(path)  // OK - splits and rejoin
git::tree_at_path(..., &path)  // Could fail on filesystem
```

### No Explicit Test Coverage

- ❌ No tests with paths exceeding 1000 characters
- ❌ No tests with 100+ nesting levels
- ❌ No error message validation for path too long errors

---

## 4. Zooming Into the Same Path Twice

### Current Status: FULLY SUPPORTED

The design explicitly supports this via the trailers system:

1. **First zoom in**: Creates a seed commit (orphan) with `git-zoom-out: path`
   trailer
2. **Zoom out**: Records the merge with `git-zoom-out: path` trailer
3. **Zoom in again**: Detects the `git-zoom-out: path` trailer and continues
   from the last subtree commit instead of creating a new seed

### No Issues Identified

- ✅ Tested by `test_multiple_zoom_cycles()` - tests zooming into same path
  multiple times
- ✅ Tested by `test_return_to_previous_zoom()` - zooms out then back in without
  specifying path
- ✅ History is preserved cleanly via first-parent lineage

### Edge Cases Within This Category: POTENTIAL ISSUE

**Explicit target with same path after divergence**:

```
Time 1: Zoom in src/lib from F3, modify, zoom out to F3 -> F7
Time 2: In full-tree, advance to F10
Time 3: Zoom in src/lib from F10 (should work)
Time 4: Zoom out with explicit target back to F3 (F3 != F10 - different commit)
```

The code handles this via the `explicit_target` parameter in zoom_out, but
there's no conflict detection. Per DESIGN.md line 296-300:

> "If the target has diverged significantly from base, consider using
> `git merge` instead of `git commit-tree` for proper three-way merge with
> conflict detection."

**Risk**: Currently uses `git commit-tree` which does a simple tree replacement,
not three-way merge. If the path was modified in both base and target commits,
changes in one might be silently lost.

### Test Coverage for This Issue

- ⚠️ `test_explicit_target()` exists but only tests case where target divergence
  is clean
- ❌ No test for conflicting modifications at the same path in base vs target

---

## 5. Files with Special Characters in Names

### Current Status: MOSTLY SAFE

The code uses git's native tools (`git ls-tree`, `git mktree`) which handle
arbitrary filenames correctly.

### Analysis

1. **UTF-8 filenames**:
   - Used throughout code: `from_utf8_lossy()` is applied
   - This means invalid UTF-8 filenames are silently converted (with lossy
     conversion)
   - Risk: Lost information on non-UTF-8 filenames (rare in modern systems)

2. **Special characters in paths**:
   - Characters like spaces, quotes, newlines, etc.
   - Git handles these in tree objects
   - Shell argument passing uses proper `Command::new()` API (safe from
     injection)

3. **Whitespace in paths**:
   - Parsing in `git.rs:ls_tree()` correctly uses tab-delimiter from git output
   - Handles spaces in filenames correctly

### Potential Issues

1. **Tab characters in filenames**:
   - `git ls-tree` output is tab-delimited
   - A filename containing a literal tab character would break parsing
   - **Example**: filename `"file\tname.txt"` would cause `split_once('\t')` to
     split incorrectly
   - **Code location**: `git.rs` lines 104-107

```rust
let (meta, name) = line.split_once('\t').ok_or_else(|| Error {
    command: "ls-tree".to_string(),
    message: format!("malformed line: {}", line),
})?;
```

2. **Newlines in filenames**:
   - `git log` output uses `\x1e` as record separator (line 404)
   - Newlines in message bodies are fine
   - But newlines in paths would appear as `%0a` in git output
   - Unknown if they're properly un-escaped

### No Explicit Test Coverage

- ❌ No tests with spaces in filenames
- ❌ No tests with newlines or tabs in filenames
- ❌ No tests with non-ASCII characters (emoji, Chinese, etc.)
- ⚠️ No tests with very long filenames (>255 characters)

---

## 6. Race Conditions and Concurrency Issues

### Current Status: PARTIALLY MITIGATED

The code uses a `DIR_MUTEX` in tests (`helpers.rs` line 16) to prevent parallel
tests from interfering.

### Analysis

1. **Process-level concurrency**:
   - The tool checks `git status --porcelain` at startup (line 62 in git.rs)
   - Requires clean working tree before any operations
   - **Risk**: Another process could modify files between check and operation
     start
   - **Mitigation**: Git operations are atomic at the commit level

2. **Within-operation races**:
   - All git operations are done via forked `git` processes
   - Git ensures atomicity of individual operations
   - No code-level races due to Rust's safety

3. **Multi-tool scenarios**:
   - If another process runs `git commit` or `git reset` while git-zoom is
     running, undefined behavior
   - No locking mechanism beyond the working tree cleanliness check
   - **Risk**: Medium - unlikely in practice but possible with concurrent git
     operations

4. **Stdin/stdout race with git subprocesses**:
   - `mktree` implementation pipes to stdin carefully (lines 127-145 in git.rs)
   - Proper handling of spawn, write, and wait_with_output
   - No identified race conditions

### Test Coverage

- ✅ Tests use DIR_MUTEX to serialize directory changes
- ❌ No explicit tests for concurrent git operations
- ❌ No tests for external process modifications during zoom

---

## 7. Very Large Commits or Trees

### Current Status: POTENTIALLY PROBLEMATIC

1. **Large trees**:
   - `git ls-tree` output parsing iterates line-by-line (git.rs line 100)
   - Each entry is stored in memory
   - No streaming or size limits

2. **Recursive tree operations**:
   - `tree.rs:replace_subtree_recursive()` is tail-recursive (in practice, not
     limited by language)
   - But the recursion depth equals path depth, not tree size
   - Tree reconstruction creates new entries vector in memory: `Vec::new()` at
     line 24

3. **Large commit messages**:
   - No explicit size limits
   - Git handles message sizes fine

### Potential Issues

1. **Out of Memory on large trees**:
   - If a repository has millions of files in a single directory, `git ls-tree`
     would return millions of lines
   - All parsed into `Vec<(String, String, String, String)>` (line 24 in
     tree.rs)
   - Could cause OOM on systems with limited RAM
   - **Example**: Directory with 10M files, 100 bytes per entry = 1GB memory

2. **Slow operations on large trees**:
   - `replace_subtree_recursive()` must reconstruct all parent directories
   - For each level of path depth, entire tree is read and recreated
   - O(n) per level where n = number of siblings
   - **Example**: Replacing `a/b/c/d/e/f` with each level having 10k files = 60k
     tree recreations

### No Test Coverage

- ❌ No tests with directories containing >1000 files
- ❌ No tests for memory usage with large trees
- ❌ No performance benchmarks for tree operations

### Example Problematic Scenario

```
Repository structure: root/ has 1M subdirectories
Operation: zoom in into root/subdir123456 (direct child)
Issue: ls-tree of root would return 1M entries, all loaded into memory
```

---

## 8. Path Normalization Edge Cases

### Current Status: MOSTLY CORRECT WITH MINOR ISSUES

`zoom_in.rs:normalize_path()` (lines 8-30) implements path normalization.

### What It Does Correctly

- Strips leading slashes: `/src/lib` → `src/lib`
- Strips trailing slashes: `src/lib/` → `src/lib`
- Removes empty components: `src//lib` → `src/lib`
- Removes `.` components: `./src/./lib` → `src/lib`
- Rejects `..` components: `src/../lib` → ERROR
- Rejects empty paths: empty string → ERROR

### Issues Identified

1. **Leading slashes handling**:
   - Path `///src/lib` gets split into empty strings which are filtered out
   - Result: `src/lib` (correct)
   - But the intent (absolute paths) is rejected silently

2. **Adjacent slashes**:
   - Path `src///lib` splits to `["src", "", "", "lib"]`
   - Empty strings filtered, result: `src/lib` (correct)
   - This is actually correct behavior but undocumented

3. **Dot-dot in middle of path**:
   - Path `src/lib/../../../etc/passwd` correctly rejects ANY `..` component
     (line 22)
   - This is correct and secure
   - But error message doesn't indicate which component was problematic

4. **Repository root rejection**:
   - Path `.` or `/` or `./` all get rejected (correct)
   - But the error message says "cannot zoom into repository root" for all
   - This is correct behavior but could be clearer

### Test Coverage

- ✅ `test_zoom_in_invalid_paths()` - tests many invalid paths
- ✅ `test_zoom_in_trailing_slash()` - tests trailing slash normalization
- ✅ `test_zoom_in_dot_normalization()` - tests `.` component removal
- ✅ `test_parse_target_path()` - tests zoom_out path parsing
- ⚠️ Limited to straightforward cases, no edge case combinations

### Undocumented Behavior

- What happens with Unicode normalization? (e.g., é vs e + combining accent)
  - Currently no normalization - git would handle inconsistently

---

## 9. Implicit Issues: Blob vs Tree Confusion

### Current Status: PARTIALLY HANDLED

In `tree.rs:replace_subtree_recursive()` (lines 37-40), if an intermediate path
component is a blob (file), it's replaced with an empty tree:

```rust
let subtree_hash = if obj_type == "blob" {
    git::empty_tree()?
} else {
    hash
};
```

### Analysis

This is intentional per DESIGN.md line 308-309:

> "If an intermediate path component is a blob (file) instead of a tree
> (directory), it is replaced with a tree."

### Potential Issues

1. **Data loss**: Original blob (file) content is lost when replaced with empty
   tree
   - **Example**: `src/lib` is a file, zooming to `src/lib/core/file.txt`
     deletes `src/lib` and replaces with tree
   - This is intentional but dramatic

2. **Symlink confusion**: Symlinks are blobs (mode 120000), would be replaced
   with trees
   - Already identified in section 2

3. **Gitlink confusion**: Git submodules (mode 160000) would also be treated as
   blobs
   - Replacing a submodule commit with empty tree would break submodule
     references

### Test Coverage

- ❌ No tests for blobs at intermediate path components
- ❌ No tests for submodules in paths
- ❌ No tests verifying blob replacement behavior

---

## 10. Trailer Parsing Edge Cases

### Current Status: MOSTLY CORRECT

`scan.rs:scan_for_trailer()` (lines 16-39) parses trailers from commit bodies.

### Implementation Details

```rust
let trailer_prefix = format!("{}: ", trailer_name);
for line in body.lines() {
    if let Some(path) = line.strip_prefix(&trailer_prefix) {
        let path = path.trim();
        if filter_path.is_some_and(|filter| path != filter) {
            continue;
        }
        return Ok(Some(Found { ... }));
    }
}
```

### Issues Identified

1. **Duplicate trailers**: Only the first matching trailer is returned
   - If commit has both `git-zoom-in: src/lib` and `git-zoom-in: src/other`,
     only first is used
   - Correct behavior but undocumented

2. **Trailer in message body vs footer**: Standard git trailers appear at the
   end after blank line
   - This code searches entire message body, including commit title and message
     text
   - **Risk**: If commit message happens to contain `git-zoom-in:` in middle of
     sentence, it's treated as trailer
   - **Example**: "We used git-zoom-in: src to isolate issues" would be parsed
     as trailer
   - This is a LOW risk since format is unusual, but possible

3. **Multi-line trailer values**: Git trailers can span multiple lines
   (continuation)
   - This code only checks individual lines
   - Multi-line trailers would not be recognized
   - **Risk**: If trailer values contain newlines (unusual), they'd be parsed as
     separate trailers

4. **Whitespace handling**:
   - Uses `.trim()` on path value after extraction (line 26)
   - Handles trailing whitespace correctly
   - Leading whitespace also trimmed

### No Test Coverage

- ❌ No tests with duplicate trailers
- ❌ No tests with trailers in message body (not footer)
- ❌ No tests with trailing whitespace in trailer values
- ❌ No tests for paths with special characters in trailers

---

## 11. Timestamp Handling

### Current Status: COMPLEX WITH POTENTIAL ISSUES

The code deterministically derives timestamps from parent commits (git.rs lines
180-187).

### Analysis

1. **Timestamp parsing**: `parse_timestamp()` (lines 234-346) handles ISO8601
   dates
   - Supports formats: `2026-01-02T21:10:36Z` and `2026-01-02T21:10:36+05:00`
   - Returns Unix timestamp for comparison
   - Includes full timezone handling

2. **Leap year handling**: `is_leap_year()` (lines 348-350) correctly implements
   leap year rules

3. **Edge cases in timestamp parsing**:

### Issues Identified

1. **Leap second handling**: Code doesn't account for leap seconds (seconds
   > 59.
   - While rare, `GIT_COMMITTER_DATE` can specify leap seconds
   - **Example**: `2026-01-02T23:59:60Z` would parse incorrectly
   - **Risk**: Very low, leap seconds are deprecated

2. **Timezone edge cases**:
   - Format `±HH:MM` is handled
   - But what about `±HH`? Code expects colon (line 301)
   - Git also requires colon, so this is correct

3. **Year precision**:
   - Uses `i32` for year parsing (line 274)
   - Year 2038 problem doesn't apply to i64 timestamps, but this is for
     comparison
   - Supports years up to 2^31-1 (year 2147483647) - fine

4. **Month/day validation**:
   - Code doesn't validate month is 1-12 or day is 1-31
   - Would silently produce incorrect timestamps for invalid dates
   - **Example**: `2026-02-30T00:00:00Z` would be parsed incorrectly but
     silently
   - **Risk**: Low - git wouldn't create invalid timestamps, but user-provided
     dates could cause issues

### No Test Coverage

- ❌ No tests for leap seconds
- ❌ No tests for invalid months/days
- ❌ No tests for year boundaries
- ❌ No tests for negative timestamps (before 1970)

---

## 12. Workflow Violations

### Current Status: NOT PROTECTED

The code doesn't prevent certain incorrect workflows:

1. **Zooming out without zooming in**:
   - `git zoom out` requires a `git-zoom-in` trailer in first-parent history
   - If user manually creates commits without proper trailers, zoom out fails
     with unclear message
   - ✅ This is correct - errors appropriately

2. **Orphaned subtrees**:
   - If a commit with `git-zoom-in` trailer is deleted from first-parent
     history, subtree can't zoom out
   - User would need to manually recreate the trailer or use explicit target

3. **Multiple interleaved zoom paths**:
   - The design supports this: different paths have independent lineages
   - First-parent history can contain mix of zoom-in/out commits for different
     paths
   - Appears to work correctly per tests

### No Issues Identified

- ✅ Design handles these cases correctly
- ✅ Error messages guide user appropriately

---

## 13. Git Configuration Assumptions

### Current Status: MOSTLY SAFE

The code calls `git` commands and relies on git configuration.

### Analysis

1. **User configuration**:
   - Committer is hardcoded: `🔎 <git-zoom-in@localhost>`
   - Author would come from git config if available
   - Fallback to zoom committer (lines 390-391 in DESIGN.md, not implemented in
     code - just uses config)
   - ⚠️ **Issue**: Code doesn't actually fall back; it just uses git's
     configured user

2. **Git hooks**:
   - Commits are created with `git commit-tree`, which bypasses hooks
   - This is intentional - git-zoom shouldn't trigger user hooks
   - **Risk**: Minimal - correct design

3. **Repository configuration**:
   - Code assumes standard git behavior
   - Unusual config (like `core.safecrlf`, `core.pager`) could cause issues
   - **Example**: If `core.pager` is configured, output might be piped through
     pager (won't happen with `--output` style captures, should be fine)

4. **Encoding configuration**:
   - No handling of `i18n.logOutputEncoding` or `i18n.commitEncoding`
   - Uses UTF-8 throughout (`from_utf8_lossy`)
   - Non-UTF-8 encoding would cause silent data loss

### No Test Coverage

- ❌ No tests with non-ASCII committer names in git config
- ❌ No tests with `core.worktree` or other repo config
- ❌ No tests with non-UTF-8 encoding

---

## 14. Precondition Checking

### Current Status: GOOD

`main.rs` and `git.rs` check preconditions:

1. ✅ Repository root check (git.rs line 42)
2. ✅ Clean working tree check (git.rs line 61)
3. ✅ HEAD exists (implicit via `git rev-parse HEAD`)

### Minor Issues

1. **Shallow clones**:
   - Git allows operations in shallow clones
   - `git-zoom` doesn't detect or warn about shallow clones
   - **Risk**: Very low - zoom should work fine in shallow clones

2. **Detached HEAD**:
   - Code works fine in detached HEAD state
   - No special handling needed
   - ✅ Correct

3. **Empty repository**:
   - First command would be `git rev-parse HEAD` (zoom_in line 52)
   - Would fail with clear error message for empty repo
   - ✅ Correct

---

## 15. Error Message Quality

### Current Status: GENERALLY CLEAR

Error messages are descriptive but have gaps:

### Good Examples

- "path 'src/lib' does not exist in HEAD" (zoom_in.rs line 59)
- "invalid path: '..' not allowed" (zoom_in.rs line 25)
- "no zoom-in commit found in history" (zoom_out.rs line 49)

### Issues

1. **Ambiguous "malformed line" error** (git.rs line 106)
   - Doesn't show the actual line that's malformed
   - User can't diagnose the problem

2. **Git command failures**:
   - Error propagates as-is from git stderr
   - Could be cryptic for git-internal errors

3. **Path too long errors**:
   - Would come from git, not descriptive about path being the issue

---

## Summary of Risk Assessment

| Category            | Risk Level | Status                | Test Coverage |
| ------------------- | ---------- | --------------------- | ------------- |
| Empty files/dirs    | Low        | ✅ Safe               | ✅ Good       |
| Symlinks            | **HIGH**   | ⚠️ Not handled        | ❌ None       |
| Long paths          | Medium     | ⚠️ Potential issue    | ❌ None       |
| Same path twice     | Low        | ✅ Safe               | ✅ Good       |
| Tab in filenames    | **HIGH**   | ⚠️ Breaks parsing     | ❌ None       |
| Race conditions     | Medium     | ⚠️ Partial mitigation | ⚠️ Limited    |
| Large trees         | Medium     | ⚠️ OOM possible       | ❌ None       |
| Path normalization  | Low        | ✅ Mostly correct     | ✅ Good       |
| Blob/tree confusion | Low        | ✅ Intentional        | ❌ Limited    |
| Trailer parsing     | Low        | ✅ Mostly correct     | ⚠️ Limited    |
| Timestamps          | Low        | ✅ Robust             | ⚠️ Limited    |
| Workflows           | Low        | ✅ Safe               | ✅ Good       |
| Git config          | Low        | ✅ Mostly safe        | ❌ None       |
| Preconditions       | Low        | ✅ Good               | ✅ Good       |
| Error messages      | Low        | ⚠️ Mostly clear       | ⚠️ Limited    |

---

## Recommendations

### High Priority

1. **Add tab character handling** in `git.rs:ls_tree()` - test with filenames
   containing tabs
2. **Add symlink handling** - either explicitly document non-support or
   implement proper handling
3. **Add error testing** for various edge cases to improve error messages

### Medium Priority

1. **Add large tree testing** - benchmark with 100k+ file directories
2. **Add long path testing** - test with 500+ character paths
3. **Add special character tests** - spaces, newlines, unicode in filenames and
   paths

### Low Priority

1. **Improve error messages** - show context for "malformed line" errors
2. **Document implicit behaviors** - clarify what happens with blobs, symlinks,
   etc.
3. **Add optional conflict detection** - use `git merge` for explicit targets
   instead of simple tree replacement
