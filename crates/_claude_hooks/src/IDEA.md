We want `.agentlock` files, which use .gitignore syntax, but specify files that
agents are forbidden from editing.

In `PreToolUse` we try to stop it directly for `Write`.

In `Stop` hook we review git history and if any of those files have lines in
them that were added by commits which were authored-by or committed-by an AI,
then we revert those files to the state they had prior to those changes.
`.agentlock` files themselves are, of course, the first thing we restore, so
agents can't edit them out (or IN -- agents cannot edit these, so agents cannot
lock themselves out of files either).
