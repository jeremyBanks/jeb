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
git zoom out
git zoom out origin/branch
git zoom out origin/branch:src/tree
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
