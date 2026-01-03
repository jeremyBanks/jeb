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

Similarly, you should almost NEVER write files under paths like `/tmp/` which
are outside of the project directory, because these also trigger user permission
prompts. You should always prefer to create test scripts within the project that
can be run using the standard tools like `cargo run` or `cargo test` or
`deno run`, even if they're only temporary and you delete them after.

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

---

This project uses `.noedit` files to indicate files and paths that can not be
directly edited by agents, and must be manually edited by humans instead. These
follow the same syntax as `.gitignore`, except that if a `.noedit` file is empty
(no content except potentially whitespace) it defaults to applying to the entire
directory it's in, as though the file contained `*`. You don't need to go out of
your way to look for these files (we have tooling that will catch accidental
disallowed edits and revert them sooner or later), but if you happen to
incidentally notice a `.noedit` file in a directory where you'll be making
changes, have a look at it to know if you're able to proceed. (Do not attempt to
edit `.noedit` files or to circumvent this policy.)

---

This project has begun experimenting with some initial tentative uses of the
`tracey` CLI tool for specification traceability annotations. You may see
`TRACEY.md` files with rule specifications `r[like.this]`, or comments in code
indicating rule implementations `[impl like.this]` or tests
`[verify like.this]`, and similar things. Most of the code doesn't use them, and
they're not used consistently, so you probably won't need to interact with them,
but we're providing this context so you understand what you're seeing, and in
case related work is requested or required. Note that the `tracey` CLI is still
in beta, and it's possible that it has bugs.
