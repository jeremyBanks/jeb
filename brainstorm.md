# Brainstorm Topics

1. Message 1
   1. Generalize the operation of squash-merging into a branch then merging the squashed back into the branch (no changes) so they're merge-compatible
   2. Figure out our actual long term workflows for git-zoom given the above and make some supporting scripts
2. Message 2
   1. Define (and use) GitHub actions taking advantage of the git zoom stuff to zoom in on our branches after every release
3. Message 3
   1. jeb-git crate which adds a git jeb subcommand with my other sub sub commands, but also has a feature flag to export multiple binaries so as to promote them to full subcommands, and git-zoom is just one of then for now
      1. Example: cargo install jeb-git —features=zoom-in,zoom-out,save,git-save-bin-save and we'll have a whole matrix controlling whether they get a git-X binary or a direct X binary, and zoom has the sub sub commands in and out so "zoom" refers to both unless specified and features that aren't required aren't compiled into any binaries and there are like "jeb-all" to enable all of them but only as sub sub commands etc
