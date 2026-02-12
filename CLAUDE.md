## This Repository

This is a fork of [jeremyBanks/jeb](https://github.com/jeremyBanks/jeb),
branched from `jeb/jeb`. We work on `mattemoon/dev`.

**We are free to do anything here.** Break things, experiment, add crates,
diverge from upstream — all fine. This is our space.

**One constraint:** we regularly merge changes FROM upstream, so keep the
overall project structure compatible enough that merges remain feasible.
Individual crates can diverge completely (we just pick one side on conflict),
but don't restructure the top-level workspace layout without good reason.

**We do NOT optimize for upstreaming.** If something gets upstreamed later,
it'll be rewritten at that point. Don't let "will this merge cleanly upstream?"
influence your work here.

**Git rules:**
- **Commit extremely often.** After every meaningful change, commit. Don't batch.
- **NEVER rewrite history** — no `--force`, no `--amend`, no `rebase -i`, no
  `reset --hard` on pushed commits. Once committed, it stays forever.
  **One exception:** if you notice *immediately* after committing (before push
  or any further commits) that you included something you didn't mean to, you
  may amend or soft-reset that single commit to fix it. That's it.
- **No force push.** Ever. If push fails, figure out why.

This repository has a messy history with bloated binary files in old commits.
Prefer a **shallow clone** (`git clone --depth=1`) if you don't need deep
history. `git fetch --unshallow` later if needed.

---

While editing files, please run `./run save` to commit all changes in the
working tree _very_ often, for the sake of having a lot of snapshots so we can
cleanly revert exactly as far as we need to, if we ever need to. You'll want to
ensure that the state before and after you make changes are both fully captured
as well as the incremental snapshots while you work.

---

_Never_ edit git history - once a commit is committed, it's never amended or
rebased or reset. If you need to roll back, do so with `git revert` or similar
commands that create new commits instead.

---

You do not need to write a descriptive commit message for every little
checkpoint commit: `./run save` will generate a placeholder commit message if
run without any arguments, and that's a common part of our workflow. But when
you're wrapping up a significant piece of work or you changed something subtle
that you want to note, `./run save` supports `-m "some message"` like
`git commit` does. It also has `--allow-empty` and `--empty` if you don't have
any changes to commit but would like to log a new message or description as a
commit message.

---

If you need to add a new reusable script to the workspace: generally, we prefer
to create scripts as Rust programs instead of bash, see
`crates/_scripts/src/bin`.

Examples that aren't specific to a single crate, or code we're sketching out and
haven't decided where to put yet, is sometimes placed in
`crates/_examples/examples`.

---

Crates that are internal-only and not meant to be published in the near future
are named with underscores (instead of hyphens) and have a leading underscore
prefix, to mark them as internal and prevent any potential clashes with real
crate names.

---

`./run` is a wrapper around `cargo run`-like behavior, but falling back to
existing builds or system installations of tools, if we aren't able to (re)build
a local copy. This is to mitigate the pain of self-hosting tools inside of our
own workspace, where changes we're making might break our ability to build the
tools we want to use while continuing to work on fixing that code!

Therefore any binary implemented in this workspace that we're running as part of
our workflow (not for the sake of developing/testing them, but to use them for
their intended purpose) should be run through `./run <command> <args...>`.
instead of `cargo run --bin <command> -- <args...>`.

YOU MUST NOT USE `./run` TO RUN A BINARY THAT YOU ARE ACTIVELY WORKING ON, FOR
TESTING! It runs in release mode, and will fall back to a previous build if
needed. During development, you DO want to be using
`cargo run --bin <command> -- <args...>` to ensure you're using a debug build
with the latest code.

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

---

When comparing image files (e.g. verifying PNG output), use the image_diff
example:

    cargo run --example image_diff -- <old> <new>

This provides a text summary of differences suitable for CI/agent use.
