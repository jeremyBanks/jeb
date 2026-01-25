# inline-ignore Algorithm

## Purpose

Copy rules from parent `.gitignore` files into a target `.gitignore`, transforming patterns as needed so they have the same effect when interpreted from the target's location.

## Gitignore Pattern Semantics (Empirically Verified)

| Pattern Type | Example | Behavior |
|--------------|---------|----------|
| Simple (no `/`) | `foo.txt`, `*.log` | Matches at **any level** |
| Anchored (`/` prefix) | `/foo.txt` | Anchored to gitignore directory |
| Path (has `/` in middle) | `a/b/foo.txt` | **Implicitly anchored** (equivalent to `/a/b/foo.txt`) |
| Directory (trailing `/` only) | `build/`, `node_modules/` | Matches directory name at **any level** |
| Anchored directory | `/build/` | Anchored to gitignore directory |
| Double-star prefix | `**/foo.txt` | Matches at any level |
| Double-star with path | `**/a/b/foo` | Matches `a/b/foo` at any level |

Key insight: `doc/frotz` and `/doc/frotz` have the same effect - a leading slash is not relevant if there is already a middle slash in the pattern.

## Transformation Rules

Given: copying from source gitignore to target, where `R` = relative path components from source to target (e.g., `["a", "b"]` for target `/a/b/`)

### Rule 1: Simple patterns (no `/` except trailing `/`)

Copy as-is. These match at any level, so they work unchanged.

```
foo.txt   →  foo.txt
*.log     →  *.log
build/    →  build/
```

### Rule 2: Anchored/path patterns (has `/` in middle, or leading `/`)

If the pattern path starts with R (the target's relative path), strip that prefix. Otherwise skip (pattern doesn't apply to target).

**Preserve original style**: only add leading `/` when semantically required (when result has no `/` in it).

```
# Target: /a/b/  (R = a/b)

a/b/c/foo   →  c/foo      (middle slash preserved, no leading / needed)
/a/b/c/foo  →  /c/foo     (preserve existing leading /)
a/b/foo     →  /foo       (MUST add / - otherwise "foo" matches any level)
/a/b/foo    →  /foo       (already has /)
b/foo       →  SKIP       (doesn't start with a/b)
/other/foo  →  SKIP       (doesn't start with /a/b)
```

### Rule 3: Double-star patterns (`**/...`)

These require **duplication** to preserve "match at any level" semantics.

For `**/P` where P contains path separators:
1. **Always keep the original** `**/P` (matches P at deeper levels within target)
2. **Check if any suffix of R is a prefix of P**: if so, add anchored pattern for the remainder

```
# Target: /a/b/  (R = a/b)

**/foo.txt      →  **/foo.txt                    (no path in P, just copy)
**/a/b/foo.txt  →  /foo.txt AND **/a/b/foo.txt   (a/b is suffix of R, prefix of P)
**/b/foo.txt    →  /foo.txt AND **/b/foo.txt     (b is suffix of R, prefix of P)
**/c/foo.txt    →  **/c/foo.txt                  (c is not suffix of R)
```

### Rule 4: Negation (`!` prefix)

Strip `!`, apply rules above, re-add `!` to all results.

### Rule 5: Trailing `/` preservation

Strip trailing `/` for processing, apply rules, re-add `/` to results.

## Line Inclusion Rules

### Comments
- Included if the line following them is copied
- When searching for insertion point, try matching comment first, then fall back to previous non-comment line

### Blank lines
- Same logic as comments
- Multiple consecutive blank lines count as a single entry

### Insertion logic
1. If identical line already exists in target → skip
2. Find the line in target matching the "previous line" from source, insert after it
3. If first line in source (no previous) → append at end of file
4. Duplicate lines within same source file are treated as distinct and all included

## Processing Multiple Parent Gitignores

Walk from target directory up to repository root. For each parent directory with a `.gitignore`:
1. Compute R (relative path components from that parent to target)
2. Transform each pattern using rules above
3. Merge results into target's gitignore using insertion rules
