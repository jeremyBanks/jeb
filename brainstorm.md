# Brainstorm Topics

1. Agent: Set up orphan branch with empty initial commit and create brainstorm.md file
2. Agent: Ensure branch is pushable - rename from brainstorm to claude/note-taking-tracker-wJTFN
3. Agent: Confirm message 1 is still in the file
4. Agent: Define instruction prefix rules - agent:, claude:, haiku:, etc. are instructions (not recorded in old policy)
5. Agent: Add more instruction prefixes - bot:, ai:, robot:, chat:
6. Agent: Display full brainstorm.md file contents
7. Generalize the operation of squash-merging into a branch then merging the squashed back into the branch (no changes) so they're merge-compatible
   1. Figure out our actual long term workflows for git-zoom given the above and make some supporting scripts
8. Define (and use) GitHub actions taking advantage of the git zoom stuff to zoom in on our branches after every release
9. jeb-git crate which adds a git jeb subcommand with my other sub sub commands, but also has a feature flag to export multiple binaries so as to promote them to full subcommands, and git-zoom is just one of then for now
   1. Example: cargo install jeb-git —features=zoom-in,zoom-out,save,git-save-bin-save and we'll have a whole matrix controlling whether they get a git-X binary or a direct X binary, and zoom has the sub sub commands in and out so "zoom" refers to both unless specified and features that aren't required aren't compiled into any binaries and there are like "jeb-all" to enable all of them but only as sub sub commands etc
10. Consider whether to update feature formatting code to put underscore-prefixed features at the bottom of the list — default behavior may be fine as-is
11. Agent: New policy - agent messages now go in the file and count toward message numbers, but are excluded from default "last two messages" output (unless pinned)
12. Agent: Recognize "unpin:" as an instruction prefix
13. Agent: Reminder - display only pinned topics and last two messages (skip instruction-only messages when finding last two, but they still count toward numbering)
14. Agent: Define "pin:" prefix - identifies messages to pin; "pin" with no target means pin last topic
15. Agent: Retroactively add all agent messages back to the file and reorder everything appropriately
16. Use cargo workspace exclude to temporarily create non-workspace versions of crates in order to reduce the lock file down to only what that crate needs, like workspace-subset-lockfile
17. Our normalization script needs to strictly enforce that all path dependencies (within our workspace) specify the same version as is actually specified in the crate's file, because that needs to match to be able to publish correctly (in most cases that will be .workspace=true so we'll need to make sure we resolve that correctly)
   1. We can use [patch.crates-io] my_lib = { path = "crates/my_lib" } in the workspace to allow the crate files to avoid specifying paths, so they work on their own
18. Reference: https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#using-patch-with-multiple-versions
19. Agent: When there are multiple non-agent topics (list items or paragraphs) in a message, they should be displayed in the summary as nested bullets
20. Since the actual name doesn't matter for overrides if we specify package=, let's always specify with a standardized naming convention
   1. Format: __serde__0_1_8_beta_3 (real name then __ before each dot, and each non-numeric character of the version replaced with an underscore)
21. In my local environment define a task or skill which fetches this branch to see my recent notes
22. Agent: That last message didn't start with agent: - it was not an instruction, it was a topic for the list
23. Update the playlist updater with ai
