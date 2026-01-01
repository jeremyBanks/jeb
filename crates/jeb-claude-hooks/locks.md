We're going to add a few Claude hooks, and they're all going to be triggered
this this crate's binary target(s).

Most important is a hook supporting our new `.agentlock` file.

This file uses the exact matching logic of .gitignore files, including relative
path resolution, and how negation works, etc.
