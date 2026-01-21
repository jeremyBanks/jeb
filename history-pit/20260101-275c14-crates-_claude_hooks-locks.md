We're going to add a few Claude hooks, and they're all going to be triggered
this this crate's binary target(s).

Most important is a hook supporting our new `.noedit` file.

This file uses the exact matching logic of .gitignore files, including relative
path resolution, and how negation works, etc. Except that as a special case, an
empty `.noedit` file matches _everything_ (i.e. it marks the entire directory
it's in and its descendants as being `noedit`). By default, `.noedit` itself is
always considered to be in `.noedit`, unless it's explicitly removed for one or
more directories using a negation like `.noedit`.

`.noedit` identifies files which _must not be edited/modified/removed by
agents_.

- In `PreToolUse`, if we get a request for `Edit` or `Write` specifying a path
  that's known to be marked `.noedit`, we block it.
- In `SessionStart`, we export the session ID as `claude
