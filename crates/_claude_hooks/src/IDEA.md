We want `.noedit` files, which use .gitignore syntax, but specify files that
agents are forbidden from editing.

This file uses the exact matching logic of .gitignore files, including relative
path resolution, and how negation works, etc. Except that as a special case, an
empty `.noedit` file matches _everything_ (i.e. it marks the entire directory
it's in and its descendants as being `noedit`). By default, `.noedit` itself is
always considered to be in `.noedit`, unless it's explicitly removed for one or
more directories using a negation like `!.noedit`.

---

In `PreToolUse` we try to stop it directly from `Write`ing to files specified in
the currently-on-disk `.noedit` files.

---

In `SessionStart` we export `JEB_CLAUDE_INITIAL_COMMIT` in `CLAUDE_ENV_FILE`,
using to the current HEAD commit ID (unless `JEB_CLAUDE_INITIAL_COMMIT` is
already defined in our env, in which case we just re-export the existing value.)

---

In `Stop` hook we are looking at changes that have been made since
`JEB_CLAUDE_INITIAL_COMMIT` (unless `HEAD` has been moved so it's no longer
`JEB_CLAUDE_INITIAL_COMMIT` or a descendant thereof, in which case we emit an
error message and quit with a bad status code.) If there's no
`JEB_CLAUDE_INITIAL_COMMIT` we also emit an error and quit with a bad status
code.

We read the `.noedit` files from `JEB_CLAUDE_INITIAL_COMMIT` (i.e. we ignore any
changes to those files from this agent.)

The goal of this hook is that we cannot allow any files specified by `.noedit`
files to have been changed. If they have been changed in either the working
directory _or_ the index, then we restore the file as it existed in
`JEB_CLAUDE_INITIAL_COMMIT` and commit that.
