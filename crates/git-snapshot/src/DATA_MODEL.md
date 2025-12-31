# Internal Data Model for git-snapshot

This document describes the in-memory Rust data structures used by the git-snapshot library. These structures are **different** from the on-disk YAML representation described in IDEA.md.

## Key Design Principles

1. **No integer commit references**: In memory, we always use full 40-character hex object IDs. Integer references are purely an on-disk serialization optimization.

2. **All defaults resolved**: The in-memory representation has all default values fully materialized. We don't track whether a value came from an explicit field or was inferred.

3. **Normalized structure**: Trees are fully expanded (not stored as deltas/patches). Each commit has its complete tree state.

4. **Type safety**: We use newtypes for object IDs, timestamps with timezones, etc. to prevent mixing up different kinds of strings.

## Core Data Types

### ObjectId

```rust
/// A git object ID (SHA-1 hash), always 40 hex characters / 20 bytes.
/// We store this as bytes internally for efficiency and to match git's internal representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId([u8; 20]);

impl ObjectId {
    /// Parse from 40-character hex string
    pub fn from_hex(s: &str) -> Result<Self, ParseError> { /* ... */ }

    /// Convert to 40-character lowercase hex string
    pub fn to_hex(&self) -> String { /* ... */ }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 20] { /* ... */ }
}
```

### Timestamp

```rust
/// A git timestamp: Unix epoch seconds with a timezone offset.
/// Git timestamps have second-level precision only (no subseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    /// Seconds since Unix epoch (must be non-negative for git compatibility)
    pub seconds: i64,

    /// Timezone offset in minutes from UTC (e.g., -120 for -02:00)
    pub offset_minutes: i16,
}

impl Timestamp {
    /// Parse from ISO 8601 string
    pub fn from_iso8601(s: &str) -> Result<Self, ParseError> { /* ... */ }

    /// Format as ISO 8601 string (e.g., "2021-01-14T08:25:36Z" or "2021-01-14T14:25:36-02:00")
    pub fn to_iso8601(&self) -> String { /* ... */ }
}
```

### Identity

```rust
/// A git identity (author or committer), consisting of name and email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Name portion (everything before the email)
    pub name: String,

    /// Email address (without angle brackets)
    pub email: String,
}

impl Identity {
    /// Parse from git format: "Name <email@example.com>"
    pub fn parse(s: &str) -> Result<Self, ParseError> { /* ... */ }

    /// Format as git string: "Name <email@example.com>"
    pub fn to_string(&self) -> String { /* ... */ }
}
```

### Tree

```rust
/// The contents of a git tree (directory).
/// This is stored as a flat map from full paths to blob contents.
/// Empty string path represents the root, which cannot contain content directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    /// Map from file paths to blob contents.
    /// Paths use forward slashes as separators, never have leading/trailing slashes.
    /// Empty tree is represented by an empty map.
    entries: BTreeMap<String, String>,
}

impl Tree {
    /// Create an empty tree
    pub fn new() -> Self { /* ... */ }

    /// Get the content of a blob at the given path
    pub fn get(&self, path: &str) -> Option<&str> { /* ... */ }

    /// Set the content of a blob at the given path (creating parent dirs as needed)
    pub fn insert(&mut self, path: String, content: String) { /* ... */ }

    /// Remove a file or directory at the given path
    /// Returns true if something was removed
    pub fn remove(&mut self, path: &str) -> bool { /* ... */ }

    /// List all paths in the tree
    pub fn paths(&self) -> impl Iterator<Item = &str> { /* ... */ }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool { /* ... */ }

    /// Apply a tree delta on top of this tree (used during deserialization)
    pub fn apply_delta(&mut self, delta: &TreeDelta) { /* ... */ }
}
```

**Design note**: We use a flat `BTreeMap<String, String>` rather than a nested tree structure for several reasons:
- Simpler to work with programmatically
- Easier to implement operations like "list all files" or "does path X exist"
- Path lookups are still O(log n)
- We can iterate in sorted path order for deterministic serialization
- Memory overhead is minimal compared to nested structures with many internal nodes

Alternative designs considered:
- Nested `HashMap<String, TreeEntry>` where `TreeEntry` is `enum { Blob(String), Tree(HashMap<...>) }`
- This would more closely mirror git's internal structure
- But it's more complex to traverse and modify
- We can always refactor to this later if needed for performance

### Commit

```rust
/// A git commit with all fields fully resolved (no defaults left unfilled).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    /// The commit's object ID
    pub id: ObjectId,

    /// Parent commit IDs (empty for root commits)
    pub parents: Vec<ObjectId>,

    /// The tree (file contents) for this commit
    pub tree: Tree,

    /// Commit author
    pub author: Identity,

    /// Author timestamp
    pub author_date: Timestamp,

    /// Committer (often same as author)
    pub committer: Identity,

    /// Commit timestamp
    pub commit_date: Timestamp,

    /// Commit message (may be empty string, but never None)
    pub message: String,
}
```

### RefName

```rust
/// A git reference name (e.g., "refs/heads/main").
/// For V1, we only support branch refs under refs/heads/.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefName(String);

impl RefName {
    /// Parse a ref name, ensuring it starts with "refs/"
    pub fn new(s: String) -> Result<Self, ParseError> { /* ... */ }

    /// Get the full ref name (e.g., "refs/heads/main")
    pub fn as_str(&self) -> &str { /* ... */ }

    /// Get the short name (e.g., "main" from "refs/heads/main")
    pub fn short_name(&self) -> &str { /* ... */ }
}
```

### HeadState

```rust
/// The state of HEAD: either pointing to a ref (symbolic) or directly to a commit (detached).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadState {
    /// HEAD points to a branch ref (e.g., "refs/heads/main")
    /// The branch may or may not exist (unborn branch if it doesn't)
    Symbolic(RefName),

    /// HEAD points directly to a commit (detached HEAD)
    Detached(ObjectId),
}
```

## Top-Level Repository Structure

```rust
/// The complete in-memory representation of a git repository snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// All commits in the repository, indexed by object ID.
    /// Only includes commits reachable from HEAD or refs.
    commits: HashMap<ObjectId, Commit>,

    /// Branch references (mapping ref names to commit IDs).
    /// For V1, these are all under refs/heads/.
    refs: BTreeMap<RefName, ObjectId>,

    /// Current HEAD state
    head: HeadState,
}

impl Repository {
    /// Create a new empty repository with an unborn HEAD
    pub fn new() -> Self { /* ... */ }

    /// Get a commit by ID
    pub fn get_commit(&self, id: &ObjectId) -> Option<&Commit> { /* ... */ }

    /// Get all commits
    pub fn commits(&self) -> impl Iterator<Item = &Commit> { /* ... */ }

    /// Get a ref's target commit ID
    pub fn get_ref(&self, name: &RefName) -> Option<&ObjectId> { /* ... */ }

    /// Get all refs
    pub fn refs(&self) -> impl Iterator<Item = (&RefName, &ObjectId)> { /* ... */ }

    /// Get the HEAD state
    pub fn head(&self) -> &HeadState { /* ... */ }

    /// Get the commit that HEAD points to (if any)
    /// Returns None for unborn HEAD
    pub fn head_commit(&self) -> Option<&Commit> { /* ... */ }

    /// Add or update a commit
    pub fn insert_commit(&mut self, commit: Commit) { /* ... */ }

    /// Add or update a ref
    pub fn insert_ref(&mut self, name: RefName, target: ObjectId) { /* ... */ }

    /// Set the HEAD state
    pub fn set_head(&mut self, head: HeadState) { /* ... */ }

    /// Remove unreachable commits (garbage collection)
    /// Called automatically after deserialization
    fn prune_unreachable(&mut self) { /* ... */ }
}
```

## Serialization-Specific Types

These types are used during serialization/deserialization but are not part of the main in-memory model:

### TreeDelta

```rust
/// A tree delta represents changes to apply on top of a base tree.
/// This is used during deserialization when parsing the on-disk format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeDelta {
    /// Replace the entire tree with new contents
    Replace(Tree),

    /// Apply modifications to the existing tree
    Modify(BTreeMap<String, TreeEntry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeEntry {
    /// Set a blob to this content
    Blob(String),

    /// Recursively modify a subtree
    Tree(BTreeMap<String, TreeEntry>),

    /// Delete this path
    Delete,
}
```

### CommitRef

```rust
/// A commit reference as it appears in the on-disk format.
/// This is converted to ObjectId when building the in-memory representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitRef {
    /// Full 40-character hex object ID
    Hex(ObjectId),

    /// Integer ID (only used during deserialization)
    Int(u32),
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid object ID: {0}")]
    InvalidObjectId(String),

    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("invalid identity: {0}")]
    InvalidIdentity(String),

    #[error("invalid ref name: {0}")]
    InvalidRefName(String),

    #[error("invalid tree entry name: {0}")]
    InvalidTreeEntryName(String),

    #[error("commit not found: {0}")]
    CommitNotFound(String),

    #[error("cycle detected in commit graph")]
    CycleDetected,

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("unexpected value type: expected {expected}, got {actual}")]
    UnexpectedType {
        expected: &'static str,
        actual: String,
    },

    #[error("missing required field: {0}")]
    MissingField(&'static str),

    #[error("unexpected field: {0}")]
    UnexpectedField(String),
}
```

## Implementation Notes

### Object ID Calculation

Note that `Commit::id` is stored in the struct, but it should be calculated from the commit's contents (parents, tree, author, dates, message) using git's standard commit hashing algorithm. We'll need to implement this calculation and ensure the stored ID matches the computed one, or compute it on the fly when needed.

For V1, since we're not interacting with real git, we could potentially use a simpler ID scheme (like sequential integers or random UUIDs converted to hex). However, using real git object IDs would make V2 integration much easier, so we should implement proper git hashing from the start.

### Tree Hashing

Similarly, git tree objects also have their own object IDs based on their contents. We might not need to store or calculate these in V1, but we should be aware that they exist in git's model. Each tree entry in git contains the mode, name, and object ID of the blob or subtree.

For V1, since we're storing trees as a flat map of path->content, we don't have tree object IDs at all. This is fine because we're not interacting with real git yet.

### Memory vs Disk Trade-offs

The in-memory representation prioritizes:
- **Simplicity**: Easy to work with programmatically
- **Type safety**: Using newtypes to prevent mistakes
- **Completeness**: All defaults are resolved, no implicit state

The on-disk representation prioritizes:
- **Compactness**: Defaults can be omitted
- **Human readability**: Trees as nested deltas, sensible ordering
- **Editability**: Easy to write by hand or generate from scripts

This is the right trade-off: the complex logic for defaults and deltas lives in the serializer/deserializer, while both the in-memory code and the human author of on-disk files get simpler interfaces.

### Performance Considerations

For V1, performance is not critical (we're targeting test fixtures, not large repos). However, some notes for potential future optimization:

- `Tree` uses `BTreeMap` for deterministic ordering, which adds log(n) overhead vs `HashMap`. For small test repos this is negligible.
- `Repository::commits` uses `HashMap` for O(1) lookups by ID, which is appropriate.
- `Repository::refs` uses `BTreeMap` for deterministic iteration order and efficient prefix operations if needed later.
- Storing `Tree` as a flat map means some operations (like "delete directory foo and everything under it") require iteration. For small trees this is fine.

### Clone and Copy

- `ObjectId` is `Copy` because it's just 20 bytes
- `Timestamp` is `Copy` because it's just two integers
- `Identity`, `Tree`, `Commit`, etc. are `Clone` but not `Copy` because they contain `String` or collections
- This allows efficient passing of IDs and timestamps while preventing accidental large copies

### Validation

The data structures should maintain certain invariants:
- `ObjectId` is always exactly 20 bytes
- `Timestamp.seconds` is non-negative (git can't represent pre-epoch times)
- `Tree` paths don't contain invalid characters (`/\:` null bytes, or equal `.`, `..`, `""`)
- `RefName` must start with `refs/`
- `Repository::commits` only contains commits reachable from HEAD or refs
- No cycles in the commit graph

These should be enforced at construction time (returning `ParseError` on invalid input) rather than using runtime assertions, so that parsing untrusted data can't panic.
