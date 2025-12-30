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
back through first-parents (expanding to fuller depth-first search is a topic
for future investigation due to performance considerations) until we find a
commit with a `git-zoom-in: path` trailer. The value of that trailer is used as
a default for the path, and the first parent of the matching commit (NOT the
commit itself) is used as a default for the commit argument. if the user
specifies a path, but no commit, then we use their path with the commit we
scanned for (error if no matching commits). if the user specifies a commit but
no path, then we use the path from the commit we scanned for (error if no
matching commit).)

in our case it scans back and see that D has the trailer, so it picks `C` as the
commit argument and src/path as the path argument, as though we'd run
`git zoom D:src/tree`.

It creates a new commit `F` whose first parent is `D`, and whose second parent
is `E`, and updates head to point to that. this way, now that we're back on the
"main" tree, all of the changes on the other tree look like a branch that was
merged in. Then we make a normal commit `E`.

```
A->B->C-------->F->E  
       \-D->E-/
```
