# IDEA-1.1: Amendments and Clarifications

This document describes amendments and clarifications to the design in IDEA.md.
These changes refine the tree representation syntax, hash formatting, and
serialization behavior.

## Hash Length in Serialized Output

When serializing commit references as hex strings (as opposed to integer IDs),
we use 8-character truncated hashes for all commits except for head commits
(those directly referenced by `refs` or `HEAD`), which use the full 40-character
hash.

When parsing, we accept hex strings between 4 and 40 characters in length. If a
truncated hash is ambiguous (matches multiple commits), that is a fatal error.

## Tree References with Special Keys

IDEA.md describes trees as mappings from string keys (filenames) to values. We
now introduce two special keys that are YAML sequences rather than strings:
`[commit]` and `[path]`. Because these are sequences (single-element arrays
containing the strings "commit" or "path"), they cannot collide with any valid
filename, which must be a string.

These special keys allow referencing content from other commits or paths,
enabling deduplication and expressing renames without repeating content.

### Syntax

```yaml
tree:
  [commit]: 1
  [path]: src/lib
  utils:
    [path]: ../shared/utils
    extra.rs: "// additional code"
  renamed.rs:
    [commit]: 2
    [path]: old-name.rs
```

### Semantics of `[commit]`

The `[commit]` key specifies which commit's tree to use as the base for this
tree or reference. Its value is a commit reference (integer or hex string).

`[commit]` is inherited through nested tree structures. If not specified at a
given level, it inherits from the parent mapping. At the root `tree` level, if
not specified, it defaults to the first parent commit.

### Semantics of `[path]`

The `[path]` key specifies which path within the `[commit]`'s tree to use as the
source. Its value is a path string.

If `[path]` is not specified, it defaults to the target path being written to
(equivalent to `.`).

Path resolution:

- If the path is `.`, or starts with `./` or `../`, it is resolved relative to
  the target path currently being written.
- Otherwise (e.g., `src/bin`), it is resolved relative to the repository root.

### References vs. Tree Definitions

A mapping value in a tree can be:

1. **A pure reference**: A mapping containing *only* `[commit]` and/or `[path]`
   (no string keys). This resolves to whatever object exists at the specified
   source location. If the source is a blob, the target becomes that blob. If
   the source is a tree, the target becomes that tree.

2. **A tree definition with modifications**: A mapping containing string keys
   (with or without `[commit]`/`[path]`). The string keys define entries in the
   tree. If `[commit]` and/or `[path]` are present, they specify a base tree
   whose entries are inherited; the string keys then represent additions,
   modifications, or deletions on top of that base.

These are not mutually exclusive in the sense that a tree definition can include
`[commit]` and `[path]` to specify its base, while also having string keys for
modifications.

### Blob References

When a reference (mapping with only special keys) resolves to a blob, it is an
error if the mapping contains any keys other than `[commit]` and `[path]`. You
cannot add entries to a blob.

```yaml
# Valid: reference to a blob
new-name.rs:
  [commit]: 1
  [path]: old-name.rs

# Error: cannot add entries to what resolves to a blob
new-name.rs:
  [commit]: 1
  [path]: old-name.rs
  something: "content"  # ERROR if old-name.rs is a blob
```

### Type Flexibility

The target path does not have an inherent type. Whatever object exists at the
source location (blob or tree) is what gets written to the target. A blob can
overwrite a tree, and a tree can overwrite a blob.

## Tree Serialization Algorithm

When serializing a commit's tree, we want to minimize redundancy by referencing
existing identical trees where possible, and by inheriting from parent trees
when the content is similar.

### Phase 1: Exact Hash Matching

First, we search for exact tree hash matches. We traverse the tree structure
breadth-first (processing trees at shallower depths before their subtrees). For
each tree, if we find an exact hash match, we can represent it as a reference
and skip processing its subtrees (since they are already covered by the match).

When searching for exact matches, we search ancestor commits in depth-first
order. Within each commit, the same hash may exist at multiple paths. We rank
candidates by path similarity to the target path, preferring matches with more
path components in common.

#### Path Similarity Scoring

Given a target path and a candidate path with a matching hash, we compute a
score as a tuple of integers, compared lexicographically:

1. The number of trailing path components that match (suffix match length)
2. Negated count of differing components at the boundary
3. Negated count of extra prefix components in the candidate

Example for target `src/bin/main.rs`:

| Candidate | Score | Explanation |
|-----------|-------|-------------|
| `src/bin/main.rs` | [3, 0, 0] | Full match of all 3 components |
| `src/bin/test.rs` | [2, -1, 0] | 2 components match, 1 differs |
| `foo/bar/src/bin/main.rs` | [3, 0, -2] | All 3 match, but 2 extra prefix |

When there are multiple ways to align the paths, choose the alignment that
produces the highest score.

### Phase 2: Inheritance for Non-Matching Trees

After phase 1, any trees that did not have an exact match need a different
strategy. For these, we check the immediate parent commits (first parent, then
second parent, etc.—not recursively through ancestors) to find one that has a
tree at the same path.

If found, we set `[commit]` to reference that parent and compute the necessary
modifications:

- Entries that exist in the parent but not in the current tree become deletions
- Entries that exist in the current tree but not the parent become additions
- Entries that differ become modifications

If no parent has a tree at that path, we serialize the tree contents directly
without a base reference.

## Testing with Fixture Files

Tests for this library should use `.yaml` fixture files rather than inline test
data. This serves both as test input and as documentation/examples.

### Test Structure

Tests should be organized with input and expected output files in a dedicated
directory (e.g., `tests/fixtures/` or `tests/snapshots/`).

The primary test pattern is:

1. Read an input `.yaml` file
2. Deserialize it to the in-memory representation
3. Serialize back to YAML
4. Compare against the expected output file

### Round-Trip Stability

After comparing against the expected output, also verify round-trip stability:
deserialize the output, serialize again, and assert the result is identical to
the first serialization. This confirms the serialized form is canonical and
stable.

### Auto-Generation of Expected Output

If an expected output file does not exist, the test should:

1. Generate the output by serializing the deserialized input
2. Write it to the expected output file path
3. Continue running (do not abort early)
4. Mark the test as failed at the end

This allows multiple missing output files to be generated in a single test run.
Git status will naturally show which files are new and need review—no special
tracking is required.
