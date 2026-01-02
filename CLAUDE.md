While editing files, please run `./scripts/git-save.sh` to commit all changes in
the working tree _very_ often, for the sake of having a lot of snapshots so we
can cleanly revert exactly as far as we need to, if we ever need to. You'll want
to ensure that the state before and after you make changes are both fully
captured as well as the incremental snapshots while you work.

---

_Never_ edit git history - once a commit is committed, it's never amended or
rebased or reset. If you need to roll back, do so with `git revert` or similar
commands that create new commits instead.

---

After you've completed a significant chunk of work, you should run
`./scripts/git-message.sh [target]` to create a merge-style commit describing
the changes. The script reads the commit message from stdin, so you can use a
heredoc:

```
./scripts/git-message.sh [target] <<'EOF'
Your message describing what changed and why...
EOF
```

The optional `[target]` argument specifies the ancestor commit that marks the
start of the work you're describing (defaults to the most recent first-parent
ancestor that is a merge commit). The script creates a merge commit where:

- The tree is unchanged (same as HEAD)
- First parent is the target commit
- Second parent is HEAD
- The message summarizes what changed between target and HEAD

This is similar to how a merged pull request appears in history - write a
message summarizing your understanding of what was changed and why, any context
or considerations you think would be useful, potentially including forward- or
backward-looking context when it's important.

---

Crates that are internal-only and not meant to be published in the near future
are named with underscores (instead of hyphens) and have a leading underscore
prefix, to mark them as internal and prevent any potential clashes with real
crate names. If we need to run these, we typically do so with
`cargo run --bin NAME --`.

---

When you're using the `Bash` tool to run a shell command directly, avoid using
bash `for` loops or similar constructs that are incompatible with Claude Code's
command allow-listing logic: these require manual re-approval every time you use
them, which is very disruptive to our intended workflows.

Similarly, avoid writing to files under paths like `/tmp/` which are outside of
the project directory, because these also trigger user permission prompts.

---

When running Rust tests, prefer to use `cargo nextest run` wherever you would
otherwise use `cargo test`. This is a backwards-compatible enhanced test runner
that supports all of the same options as cargo test, but with more helpful
output and options for more nuanced control of test execution.

---

If you are unable to find an expected binary, it might be missing from your
`PATH` due to bugs or sandboxing. Before anything else, you may want to check if
you can run it from `~/.cargo/bin/$NAME` explicitly, and just do that if it
works.
