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
- **F8** (bridge): parent=S7, tree=F3's tree with `src/tree` replaced by S7's tree
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

| Commit | Trailer | Purpose |
|--------|---------|---------|
| S6, S12 | `git-zoom-in: src/tree` | Found when scanning subtree to zoom out |
| F9 | `git-zoom-out: src/tree` | Found when scanning full-tree to zoom in |

Bridge commits (S4, S11, F8) may also have trailers for debugging, but these
are not used for scanning since they're not in first-parent history.

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

### Edge cases

**Multiple paths**: Each path gets its own seed commit and lineage. Scanning
filters by path.

**Sub-sub-trees**: Zooming in from a subtree works the same way. Creates a new
seed and lineage. Zooming out traverses back up.

**Multiple zoom-outs in a row**: Error unless explicit target specified. No
`git-zoom-in` trailer exists in a lineage that was already zoomed out from.

**Multiple zoom-ins in a row**: Error unless targeting different paths. No
`git-zoom-out` trailer exists in a fresh subtree lineage.

### Committer identity

Commits created by git-zoom use special committer identities:
- Zoom in: `🔎 <git-zoom-in@localhost>`
- Zoom out: `🔍 <git-zoom-out@localhost>`

Author is preserved from git config (or uses the zoom identity if none set).

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
S4:
Message: Zoom in to 'src/tree'
Committer: 🔎 <git-zoom-in@localhost>

S5:
Message: Initial commit
Committer: 🔎 <git-zoom-in@localhost>

S6:
Message: Merge from tree 'src/tree'
Committer: 🔎 <git-zoom-in@localhost>


Message: Zoom out from 'src/tree'
Committer: 🔍 <git-zoom-out@localhost>

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
