While editing files, please use `git save` (a custom alias which takes no
arguments) to commit all changes in the working tree _very_ often, for the sake
of having a lot of snapshots so we can cleanly revert exactly as far as we need
to, if we ever need to. You'll want to ensure that the state before and after
you make changes are both fully captured as well as the incremental snapshots
while you work. (If `git save` is not defined, look at the definition in
`./.devcontainer/post-create.sh` and use `git config` to define it for this
repository.)

---

_Never_ edit git history - once a commit is committed, it's never amended or
rebased or reset. If you need to roll back, do so with `git revert` or similar
commands that create new commits instead.

---

After you've completed a significant chunk of work, you should use

```
git commit --allow-empty --trailer "Co-Authored-By: Claude Code <noreply@anthropic.com>" -m "$(cat <<'EOF'
...
EOF
)"
```

to write a commit message summarizing your understanding of what was changed and
why, any context or considerations you think would be useful, potentially
including forward- or backward-looking context when it's important, sort-of like
we might do for a high-quality pull request description.

Similar to a merged pull request, you should use `git commit-tree` (with the
existing HEAD's tree) so that the HEAD is the second parent, and the first
parent is the commit _before_ the first commit you're describing, so it's like
you're merging in that history branch and describing it, then saves it with
`git update-ref HEAD`.

---
