# git-zoom design (or maybe just brainstorming)

This describes the planned design for `git-zoom`, a new command line tool (to be
used as a `git` subcommand) for extracting and embedding sub-trees within larger
git repositories.

This draws inspiration from `git-subtree` and enables some similar workflows,
but this is more limited, which lets it be much simpler to implement and
understand.

## Sub-sub commands

The subcommand is actually two sub-sub-commands, with optional arguments:

```
git zoom in
git zoom in src/tree
git zoom in src/tree --allow-empty
git zoom out
git zoom out origin/branch
git zoom out origin/branch:src/tree
git zoom out origin/branch:src/tree --deny-empty
```

## Purpose and behavior

### Goal

Enable working on a subtree of a repository as if it were its own independent
repository, then merging changes back. Both the full-tree and subtree lineages
should have clean first-parent histories, and merge commits should have
comparable parent trees for sensible diffs.

### Commit structure

Each zoom operation creates **two commits**:

1. **Bridge commit**: Converts tree type (full→subtree or subtree→full), parent
   is on the "source" lineage
2. **Merge commit**: Merges the bridge into the "destination" lineage as second
   parent, preserving first-parent lineage

Additionally, the **first zoom-in to a new path** creates a **seed commit**: an
orphan commit with an empty tree that becomes the root of the subtree's
first-parent lineage. This ensures the subtree history never includes full-tree
commits in its first-parent traversal.

### Example

```
F1->F2->F3-------------------->F9->F10-----------> full-tree lineage
         \-S4---\       /-F8-/       \-S11-\
              S5->S6->S7------------------->S12--> subtree lineage
```

**First zoom in** (`git zoom in src/tree` at F3):

- **S4** (bridge): parent=F3, tree=F3's subtree at `src/tree`
- **S5** (seed): no parent, empty tree
- **S6** (merge): first-parent=S5, second-parent=S4, tree=S4's tree

**Work on subtree**: S6→S7

**Zoom out** (`git zoom out` at S7):

- **F8** (bridge): parent=S7, tree=F3's tree with `src/tree` replaced by S7's
  tree
- **F9** (merge): first-parent=F3, second-parent=F8, tree=F8's tree

**Work on full tree**: F9→F10

**Zoom back in** (`git zoom in` at F10):

- **S11** (bridge): parent=F10, tree=F10's subtree at `src/tree`
- **S12** (merge): first-parent=S7, second-parent=S11, tree=S11's tree

### First-parent histories

- **Full-tree**: F1→F2→F3→F9→F10→... (only full-tree commits)
- **Subtree**: S5→S6→S7→S12→... (only subtree commits, no F commits)

The seed commit (S5) breaks the link to full-tree history while preserving
traceability via second-parent.

### Trailers

Trailers are placed on the **merge commits** (which appear in first-parent
history) to enable scanning:

| Commit  | Trailer                  | Purpose                                  |
| ------- | ------------------------ | ---------------------------------------- |
| S6, S12 | `git-zoom-in: src/tree`  | Found when scanning subtree to zoom out  |
| F9      | `git-zoom-out: src/tree` | Found when scanning full-tree to zoom in |

Bridge commits (S4, S11, F8) may also have trailers for debugging, but these are
not used for scanning since they're not in first-parent history.

### Argument resolution

**`git zoom in [path]`**:

- If path specified: use that path, create new seed if no prior history for path
- If no path: scan first-parent for `git-zoom-out` trailer, use its path
- Error if no path specified and no `git-zoom-out` found

**`git zoom out [target[:path]]`**:

- Scan first-parent for `git-zoom-in` trailer (filtered by path if specified)
- Default target: first-parent of the bridge commit from the found merge
- Default path: path from the found trailer
- If explicit target given: use that commit as merge first-parent
- If explicit path given: scan for trailer matching that path
- Error if no matching `git-zoom-in` found

### Multiple paths and nested trees

Each path gets its own seed commit and independent lineage. Scanning always
filters by path when a path is specified.

**Nested subtrees**: When zooming in from a subtree, paths are always relative
to the current tree root. If you're zoomed into `src/` and then zoom into
`lib/`, the trailer records `git-zoom-in: lib` (not `src/lib`). Each zoom level
is independent — zooming out from the nested subtree returns you to the parent
subtree, and zooming out again returns to the full tree.

### Not branch-aware

This tool operates purely on commits and trailers, not branch references. When
scanning for trailers, it walks first-parent history from HEAD. It has no
knowledge of remote branches or other refs.

When zooming out, the default target is the commit you originally zoomed in from
(found via trailer), not "the current tip of some branch." If others have pushed
commits to a shared branch, you'll need to merge separately or specify an
explicit target.

### Committer identity

Commits created by git-zoom use special committer identities:

- Zoom in: `🔎 <git-zoom-in@localhost>`
- Zoom out: `🔍 <git-zoom-out@localhost>`

Author is preserved from git config (or uses the zoom identity if none set).

## Implementation plan

### Technology choices

- **Language**: Rust, used as a robust scripting/glue language
- **Dependencies**: Minimal - just std library, maybe `clap` for arg parsing
- **Git interaction**: Shell out to the `git` executable for all operations

### Project structure

```
src/
  main.rs          # Entry point, arg parsing, dispatch to zoom_in/zoom_out
  git.rs           # Wrapper functions for git commands
  zoom_in.rs       # git zoom in implementation
  zoom_out.rs      # git zoom out implementation
  scan.rs          # First-parent history scanning for trailers
  tree.rs          # Tree manipulation (extract subtree, replace subtree)
```

### Preconditions (checked on startup)

1. Must be run from git repository root (`git rev-parse --show-toplevel`)
2. Working tree must be clean (`git status --porcelain` is empty)
3. HEAD must exist (not an empty repository)

### Git commands used

| Operation                | Command                                                      |
| ------------------------ | ------------------------------------------------------------ |
| Check repo root          | `git rev-parse --show-toplevel`                              |
| Get HEAD commit          | `git rev-parse HEAD`                                         |
| Check working tree clean | `git status --porcelain`                                     |
| Get tree at path         | `git rev-parse <commit>:<path>`                              |
| List tree entries        | `git ls-tree <tree>`                                         |
| Create tree object       | `git mktree` (stdin: ls-tree format lines)                   |
| Create commit            | `git commit-tree <tree> -p <parent> [-p <parent2>] -m <msg>` |
| Update HEAD              | `git update-ref HEAD <sha>`                                  |
| Reset working tree       | `git reset --hard HEAD`                                      |
| Scan history             | `git log --first-parent --format='%H %B' HEAD`               |
| Get user config          | `git config user.name`, `git config user.email`              |

### Algorithm: `git zoom in [path]`

```
fn zoom_in(path: Option<String>, allow_empty: bool) -> Result<()>:
    # 1. Resolve path and find existing subtree history
    zoom_out_found = None

    if path is None:
        # No path specified: must find it from a previous zoom-out
        zoom_out_found = scan_first_parent_for_trailer("git-zoom-out")
        if zoom_out_found is None:
            error("No path specified and no previous zoom-out found")
        target_path = zoom_out_found.path
    else:
        target_path = path
        # Check if we have previous zoom-out for this path (to continue existing lineage)
        zoom_out_found = scan_first_parent_for_trailer("git-zoom-out", filter_path=target_path)

    # 2. Check path exists in HEAD
    subtree_hash = git_rev_parse(f"HEAD:{target_path}")
    if subtree_hash is error:
        if not allow_empty:
            error(f"Path '{target_path}' does not exist in HEAD")
        subtree_hash = git_mktree("")  # empty tree

    # 3. Get current HEAD
    head_commit = git_rev_parse("HEAD")

    # 4. Create bridge commit (S4 or S11)
    bridge_msg = f"Zoom in to '{target_path}'\n\ngit-zoom-bridge: {target_path}"
    bridge_commit = git_commit_tree(
        tree=subtree_hash,
        parents=[head_commit],
        message=bridge_msg,
        committer="🔎 <git-zoom-in@localhost>"
    )

    # 5. Determine merge first parent
    if zoom_out_found is None:
        # Fresh subtree: create orphan seed commit
        empty_tree = git_mktree("")
        seed_msg = f"Initial commit for '{target_path}'"
        seed_commit = git_commit_tree(
            tree=empty_tree,
            parents=[],  # orphan
            message=seed_msg,
            committer="🔎 <git-zoom-in@localhost>"
        )
        merge_first_parent = seed_commit
    else:
        # Existing subtree: continue from where we last zoomed out
        # zoom_out_found.commit = F9 (merge with git-zoom-out trailer)
        # zoom_out_found.second_parent = F8 (bridge commit)
        # F8's parent = S7 (last subtree commit)
        last_subtree_commit = git_rev_parse(f"{zoom_out_found.second_parent}^")
        merge_first_parent = last_subtree_commit

    # 6. Create merge commit (S6 or S12)
    merge_msg = f"Merge from '{target_path}'\n\ngit-zoom-in: {target_path}"
    merge_commit = git_commit_tree(
        tree=subtree_hash,
        parents=[merge_first_parent, bridge_commit],
        message=merge_msg,
        committer="🔎 <git-zoom-in@localhost>"
    )

    # 7. Update HEAD and reset
    git_update_ref("HEAD", merge_commit)
    git_reset_hard()
```

### Algorithm: `git zoom out [target[:path]]`

```
fn zoom_out(target_and_path: Option<String>, deny_empty: bool) -> Result<()>:
    # 1. Parse target and path from argument
    (explicit_target, explicit_path) = parse_target_path(target_and_path)

    # 2. Scan for git-zoom-in trailer (filtered by path if specified)
    found = scan_first_parent_for_trailer("git-zoom-in", filter_path=explicit_path)
    if found is None:
        error("No zoom-in commit found in history" +
              (f" for path '{explicit_path}'" if explicit_path else ""))

    # 3. Determine path
    path = explicit_path or found.path

    # 4. Determine base commit (where we zoomed in from — used for tree construction)
    # found.commit = S6 or S12 (merge commit with git-zoom-in trailer)
    # found.second_parent = S4 or S11 (bridge commit)
    # bridge's parent = F3 or F10 (full-tree commit we zoomed in from)
    zoom_in_bridge = found.second_parent
    base_commit = git_rev_parse(f"{zoom_in_bridge}^")

    # 5. Determine target commit (where to merge into full-tree lineage)
    # By default, this equals base_commit. With explicit target, it can differ.
    target_commit = explicit_target ? git_rev_parse(explicit_target) : base_commit

    # 6. Get current HEAD tree (subtree content)
    head_commit = git_rev_parse("HEAD")
    head_tree = git_rev_parse("HEAD^{tree}")

    # 7. Check for empty subtree if --deny-empty
    if deny_empty and tree_is_empty(head_tree):
        error("Subtree is empty (use without --deny-empty to allow)")

    # 8. Build full tree: BASE's tree with path replaced by head's tree
    # We use base_commit (not target_commit) so that the diff base→bridge
    # shows exactly the subtree changes. If target differs from base,
    # git merge will handle the three-way merge.
    base_tree = git_rev_parse(f"{base_commit}^{{tree}}")
    new_full_tree = replace_subtree(base_tree, path, head_tree)

    # 9. Create bridge commit (F8)
    bridge_msg = f"Zoom out from '{path}'\n\ngit-zoom-bridge: {path}"
    bridge_commit = git_commit_tree(
        tree=new_full_tree,
        parents=[head_commit],
        message=bridge_msg,
        committer="🔍 <git-zoom-out@localhost>"
    )

    # 10. Create merge commit (F9)
    # If target == base, this is a simple fast-forward-style merge.
    # If target != base, the merge may need conflict resolution.
    merge_msg = f"Merge to '{path}'\n\ngit-zoom-out: {path}"
    merge_commit = git_commit_tree(
        tree=new_full_tree,
        parents=[target_commit, bridge_commit],
        message=merge_msg,
        committer="🔍 <git-zoom-out@localhost>"
    )

    # 11. Update HEAD and reset
    git_update_ref("HEAD", merge_commit)
    git_reset_hard()
```

**Note on explicit targets**: When `target_commit != base_commit`, the bridge
commit's tree is still based on `base_commit` (where you zoomed in from). This
ensures the diff `base→bridge` cleanly shows your subtree changes. If the target
has diverged from base, consider using `git merge` instead of `git commit-tree`
for step 10 to get proper three-way merge with conflict detection.

### Tree manipulation: `replace_subtree(base_tree, path, new_subtree)`

This replaces a subtree at a given path within a tree. The path can be nested
(e.g., `src/lib/core`), requiring recursive tree reconstruction.

**Blob replacement**: If an intermediate path component is a blob (file) instead
of a tree (directory), it is replaced with a tree. This allows zooming into
paths that previously didn't exist or were files.

```
fn replace_subtree(base_tree: TreeHash, path: &str, new_subtree: TreeHash) -> TreeHash:
    parts = path.split('/')
    return replace_subtree_recursive(base_tree, parts, new_subtree)

fn replace_subtree_recursive(tree: TreeHash, path_parts: &[&str], new_subtree: TreeHash) -> TreeHash:
    if path_parts.is_empty():
        return new_subtree

    target_name = path_parts[0]
    remaining_path = &path_parts[1..]

    # List current tree entries
    entries = git_ls_tree(tree)  # Vec<(mode, type, hash, name)>

    # Build new entries, replacing/adding the target
    new_entries = []
    found = false

    for (mode, obj_type, hash, name) in entries:
        if name == target_name:
            found = true
            if remaining_path.is_empty():
                # Replace this entry entirely with new_subtree
                new_entries.push(("040000", "tree", new_subtree, name))
            else:
                # Need to recurse; if existing entry is a blob, replace with empty tree first
                if obj_type == "blob":
                    hash = git_mktree("")  # Replace blob with empty tree
                new_hash = replace_subtree_recursive(hash, remaining_path, new_subtree)
                new_entries.push(("040000", "tree", new_hash, name))
        else:
            new_entries.push((mode, obj_type, hash, name))

    if not found:
        # Need to create new entry (and possibly intermediate trees)
        if remaining_path.is_empty():
            new_entries.push(("040000", "tree", new_subtree, target_name))
        else:
            # Create empty tree, recurse, then add
            empty_tree = git_mktree("")
            new_hash = replace_subtree_recursive(empty_tree, remaining_path, new_subtree)
            new_entries.push(("040000", "tree", new_hash, target_name))

    # Create new tree from entries
    return git_mktree(entries_to_ls_tree_format(new_entries))
```

### History scanning: `scan_first_parent_for_trailer`

```
fn scan_first_parent_for_trailer(trailer_name: &str, filter_path: Option<&str>) -> Option<Found>:
    # Use git log to walk first-parent history
    output = git_log("--first-parent", "--format=%H%x00%P%x00%B%x00", "HEAD")

    for each commit in parse_log_output(output):
        commit_hash = commit.hash
        parents = commit.parents.split(' ')
        body = commit.body

        # Look for trailer in body
        for line in body.lines():
            if line.starts_with(f"{trailer_name}: "):
                path = line.strip_prefix(f"{trailer_name}: ")
                if filter_path is None or path == filter_path:
                    return Found {
                        commit: commit_hash,
                        path: path,
                        second_parent: parents[1] if len(parents) > 1 else None,
                    }

    return None
```

### Committer handling

```
fn make_commit_with_committer(tree, parents, message, zoom_committer) -> CommitHash:
    # Get author from git config (or use zoom identity as fallback)
    author_name = git_config("user.name") or zoom_committer.name
    author_email = git_config("user.email") or zoom_committer.email

    # Set environment for commit-tree
    env = {
        "GIT_AUTHOR_NAME": author_name,
        "GIT_AUTHOR_EMAIL": author_email,
        "GIT_COMMITTER_NAME": zoom_committer.name,
        "GIT_COMMITTER_EMAIL": zoom_committer.email,
    }

    return git_commit_tree(tree, parents, message, env)
```

### Error handling

All git commands should:

1. Check exit code, fail fast on non-zero
2. Capture stderr for error messages
3. Provide context about what operation failed

Specific error cases:

- Path doesn't exist: "Path 'src/foo' does not exist in HEAD"
- No trailer found: "No zoom-in found in history (are you on a subtree?)"
- Working tree dirty: "Working tree has uncommitted changes"
- Not in repo: "Not in a git repository"

### Flags

| Flag            | Command  | Effect                                                       |
| --------------- | -------- | ------------------------------------------------------------ |
| `--allow-empty` | zoom in  | Allow zooming into non-existent path (creates empty subtree) |
| `--deny-empty`  | zoom out | Error if subtree would be empty                              |

### Testing strategy

1. **Unit tests**: Tree manipulation functions with mock git commands
2. **Integration tests**: Full zoom in/out cycles in temp git repos
3. **Test cases**:
   - Basic zoom in, make changes, zoom out
   - Multiple zoom cycles
   - Different paths
   - Sub-sub-trees
   - Explicit target commits
   - Error cases (dirty tree, no trailer, missing path)

## Implementation notes

This will be implemented in Rust, but all of the git operations are going to be
performed by calling the real `git` executable. The Rust program should have
very few or no dependencies, and be kept as simple as practical. We're
essentially using Rust as a more robust reliable scripting/glue language, not
performing architecture astronauting.

Exit immediately if not run from the root of a git repository. (This way there's
no ambiguity about whether paths are relative to the root or the current
directory, and we avoid the current directory disappearing from under us while
we're running.)

---

# rough notes to read and capture into the document above

DO NOT DELETE THESE BUT THEY MAY BE WRONG FYI BUT PRESERVE THEM, DO NOT EDIT OR
DELETE.

## example

We're in a repo with commits

```
A->B->C  # full tree commits
```

we run `git zoom in src/tree`

this creates a new commit `D`, whose root tree is the tree that was at src/tree
in `C`. commit `D` has the trailer `git-zoom-in: src/tree`. then we make some
changes in a commit `E`.

```
       /->D->E # sub tree commits
A->B->C        # full tree commits
```

now we run `git zoom out`.

(argument handling: Because we didn't specify a commit ref or a path, we scan
back through first-parents (whenever we talk about scanning through history in
this document assume we mean only first parents - we might revisit that in the
future, so in our doc above we'll want to use some phrasing like "in the initial
version" to describe the first-parent behavior but don't use any language that
implies anything else either, although we will want to think about whether
depth-first might actually do some nice things) until we find a commit with a
`git-zoom-in` trailer. The value of that trailer is used as a default for the
path, and the first parent of the matching commit (NOT the commit itself) is
used as a default for the commit argument. if the user specifies a path, but no
commit, then we use their path with the commit we scanned for (error if no
matching commits). if the user specifies a commit but no path, then we use the
path from the commit we scanned for (error if no matching commit).)

in our case it scans back and see that D has the trailer, so it picks `C` as the
commit argument and src/path as the path argument, as though we'd run
`git zoom D:src/tree`.

It creates a new commit `F` whose first parent is `C`, and whose second parent
is `E`, and updates head to point to that. this way, now that we're back on the
"main" tree, all of the changes on the other tree look like a branch that was
merged in. It gets a `git-zoom-out: src/tree` trailer. Then we make a normal
commit `G`.

```
A->B->C-------->F->G  # full tree commits
       \-D->E-/       # sub tree commits
```

now we do `git zoom in` again, and it's a bit more interesting because now we
have some some `git-zoom-out` commits in our history. So we can and find `E`.
Because we've specified nothing, we re-zoom on what we most-recently
zoomed-out-of. However, in addition to that, we use the `git-zoom-out` commit we
found _as the first parent_ with `HEAD` as the second parent. (If we had
specified a path, like we did the first time, we'd have scanned back for commits
with a trailer _that matched that path_.) This way, while we're on a branch
whose HEAD is in the sub tree, all of the changes from the full tree look like
merges into our own tree, so if we end up with these in different branches or
different repos, zooming out will look a lot like merging changes from upstream
(because of how tools privilege the first-parent lineage). This is commit `H`,
then we make a normal commit `I`. (Our initial commits `A`, `B`, `C` are in the
first-parent lineage for both, of course, that's probably fine, unless we want
to do something truly absurd like create an empty seed commit for each new
path... we won't do that for this example, though.)

```
       /-D->E-\------->H->I  # sub tree commits
A->B->C-------->F->G-/       # full tree commits
```

It's possible that we might want to split up the zoom-out and the merge into
separate commits for the sake of easier git tool handling.

actually we need to do that both ways to get our clean histories.

```
F1->F2->F3-------------------->F9->F10-----------> full commit branch/view
         \-S4---\       /-F8-/       \-S11-\
              S5->S6->S7------------------->S12--> sub tree branch/view
```

```
S4, S11:
Message: Zoom in to 'src/tree'
Committer: 🔎 <git-zoom-in@localhost>

S5:
Message: Initial commit
Committer: 🔎 <git-zoom-in@localhost>

S6, S12:
Message: Merge from tree 'src/tree'
Committer: 🔎 <git-zoom-in@localhost>

F8:
Message: Zoom out from 'src/tree'
Committer: 🔍 <git-zoom-out@localhost>

F9:
Message: Merge to tree 'src/tree'
Committer: 🔍 <git-zoom-out@localhost>
```

(We only set the committer, we use the default author... unless there is no
author set, in which case we use our own value for the author too, instead of
git's meaningless defaults.)

We could imagine different sub-trees branching off of the full-tree, or
sub-sub-trees, or zooming out to embed ourselves into another repository we
previously had no connection to, or zooming in and out of different parents, and
this model should be able to do the right thing, if we get the details right.

or actually, this doesn't even need to be a fake merge commit - we could
literally actually invoke git merge? but then if it fails we're in trouble
because we don't want to have to be able to resume our own logic after the user
handles a merge commit... but if that merge commit is the last thing that's
happening, then there's no need to resume so maybe it would be fine? If we could
set this up so there are _real_ merge commits using `git-merge` and we're not
just constructing that history ourselves, that would be great.

---

## Design notes and future considerations

### Using real git merge

The merge commits created by git-zoom could potentially use `git merge` instead
of `git commit-tree`. This would enable:

- Proper three-way merge when zooming out to an explicit target that differs
  from the base commit
- Automatic conflict detection if the subtree path was modified on both lineages
- Standard git conflict resolution workflow

The bridge commit is always created first (deterministic, no conflicts). Then if
the merge commit is the last operation, we could invoke `git merge` and let git
handle conflicts. If the merge fails, the user resolves conflicts normally.

For the default case (zooming out to the same commit we zoomed in from), there
are no conflicts possible — the merge is trivially resolved.

### History scanning strategy

Currently we scan first-parent history only. Future versions might consider:

- Depth-first traversal for finding trailers
- Handling octopus merges (commits with >2 parents)
- More sophisticated path matching for complex workflows
