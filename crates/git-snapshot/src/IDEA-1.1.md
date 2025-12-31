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

When serializing, special keys sort before string keys in the mapping output.

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

Like `[commit]`, `[path]` is inherited through nested tree structures, but in a
relative way: each level appends its entry name to the parent's effective source
path. This creates a mapping between target paths (where we're writing in the
new tree) and source paths (where we're reading from in the referenced commit).

**Computing the effective source path:**

1. **At the root `tree` level**: If `[path]` is not specified, it defaults to
   `.` (the root of the commit's tree). This means source path = target path by
   default.

2. **At nested levels without explicit `[path]`**: The effective source path is
   the parent's effective source path plus this entry's name. This extends the
   mapping naturally through the tree.

3. **When `[path]` is explicitly specified**:
   - If the path is `.`, or starts with `./` or `../`, it is resolved relative
     to the source path that would be computed by inheritance (i.e., parent's
     effective source path plus this entry's name).
   - Otherwise (e.g., `src/bin`), it is resolved relative to the repository
     root.
   - Resolving `..` past the repository root is an error.

**Example of path inheritance:**

```yaml
tree:
  strange-name:
    [path]: foo/src # target: strange-name → source: foo/src
    bins: # target: strange-name/bins → source: foo/src/bins
      main.rs: "use foo..."
      test: # target: strange-name/bins/test → source: foo/src/bins/test
        main.test.rs: "..." # target: strange-name/bins/test/main.test.rs
#      → source: foo/src/bins/test/main.test.rs
```

Here, setting `[path]: foo/src` on `strange-name` remaps that subtree. All
descendants inherit the remapped source path, so `strange-name/bins/test`
sources from `foo/src/bins/test` in the referenced commit.

**Default behavior matches IDEA.md:**

When no special keys are used, the implicit defaults are:

```yaml
tree:
  [commit]: <first-parent>
  [path]: .
```

This means "inherit the entire tree from the first parent commit, with source
paths equal to target paths." Any string keys then represent modifications on
top of that base, exactly as described in IDEA.md.

### References vs. Tree Definitions

A mapping value in a tree can be:

1. **A pure reference**: A mapping containing _only_ `[commit]` and/or `[path]`
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

In IDEA.md, blobs are always represented as strings (their content). IDEA-1.1
extends this: a blob can also be represented as a mapping containing only
`[commit]` and/or `[path]` keys, which references a blob from another location.
The "physical" blob (where content actually appears in the document) is always a
string; blob references are always mappings with only special keys.

When a reference resolves to a blob, it is an error if the mapping contains any
keys other than `[commit]` and `[path]`. You cannot add entries to a blob.

```yaml
# Valid: reference to a blob
new-name.rs:
  [commit]: 1
  [path]: old-name.rs

# Error: cannot add entries to what resolves to a blob
new-name.rs:
  [commit]: 1
  [path]: old-name.rs
  something: "content" # ERROR if old-name.rs is a blob
```

Invalid states (such as string keys on a mapping that resolves to a blob) are
errors. This document specifies what is valid; implementations should validate
inputs appropriately.

### Type Flexibility

The target path does not have an inherent type. Whatever object exists at the
source location (blob or tree) is what gets written to the target. A blob can
overwrite a tree, and a tree can overwrite a blob.

## Serialization Algorithm for Trees and Blobs

When serializing a repository, we want to minimize redundancy. Each unique blob
or tree hash should have its content appear exactly once in the serialized
document. All other occurrences become references.

### Physical vs. Logical References

**Physical appearance**: The actual content of a blob or tree appears exactly
once in the document—at its first occurrence in document order. This is where
the content "physically lives."

**Logical references**: When serializing a commit, we create references that
point to other commits. These references prefer to follow the commit's own
history, creating natural chains: commit 3 references commit 2, which references
commit 1 (where the content physically lives). This makes the references
meaningful in terms of git history.

**Reference chains must terminate**: Every chain of references must eventually
reach a physical location where the content actually appears. Usually the
ancestor reference is also the physical location, but not always (e.g., when
content appears on parallel branches).

### Traversal and Matching

We traverse the tree structure depth-first. For each blob or tree we encounter:

1. **Check if already serialized**: If this exact hash has already been written
   to the document (at any path, in any commit), we must emit a reference
   instead of repeating the content.

2. **Search ancestors for a reference target**: Search the current commit's
   ancestors depth-first for a matching hash. If found, reference that ancestor.
   This creates history-following reference chains.

3. **Fallback to physical location**: If the hash exists in the document but not
   in this commit's ancestry (e.g., it appeared on a parallel branch), reference
   the commit where it physically appears in the document.

4. **First occurrence**: If this is the first time we've seen this hash, write
   the content directly. This becomes the physical location.

When a tree has an exact match, we do not need to process its subtrees
separately—they are covered by the tree reference.

### Path Similarity Scoring

When multiple paths contain the same hash (either within a single commit or
across commits), we choose which one to reference based on path similarity to
the target path.

Given a target path and a candidate path with a matching hash, we compute a
score as a tuple of integers, compared lexicographically:

1. The number of trailing path components that match (suffix match length)
2. Negated count of differing components at the boundary
3. Negated count of extra prefix components in the candidate

Example for target `src/bin/main.rs`:

| Candidate                 | Score      | Explanation                     |
| ------------------------- | ---------- | ------------------------------- |
| `src/bin/main.rs`         | [3, 0, 0]  | Full match of all 3 components  |
| `src/bin/test.rs`         | [2, -1, 0] | 2 components match, 1 differs   |
| `foo/bar/src/bin/main.rs` | [3, 0, -2] | All 3 match, but 2 extra prefix |

When there are multiple ways to align the paths, choose the alignment that
produces the highest score. When the scores are identical, break the tie by path
lexicographically.

This scoring applies both when searching ancestors and when falling back to
physical locations.

### Inheritance for Modified Trees

For trees that do not have an exact hash match anywhere, we use inheritance to
minimize the diff. We check the immediate parent commits (first parent, then
second parent, etc.—not recursively through ancestors) to find one that has a
tree at the same path.

If found, we set `[commit]` to reference that parent and compute the necessary
modifications based on this new default/base tree.

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
the first serialization. This confirms the serializer produces stable,
deterministic output.

### Auto-Generation of Expected Output

If an expected output file does not exist, the test should:

1. Generate the output by serializing the deserialized input
2. Write it to the expected output file path
3. Continue running (do not abort early)
4. Mark the test as failed at the end

This allows multiple missing output files to be generated in a single test run.
