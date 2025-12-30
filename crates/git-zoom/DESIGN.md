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

[TODO: write everything important]

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

# rough notes to read and capture into the document above then delete

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
