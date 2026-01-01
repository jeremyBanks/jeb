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
We also export `JEB_CLAUDE_SESSION_ID` with the current session ID
(unconditionally) and `JEB_CLAUDE_SESSION_ID_INITIAL` (only if it's not already
present, if it is already in our env then we re-export it).

We update scripts/git-save.sh and scripts/git-message.sh to include
`Agent-Session-ID` trailers with the value of `JEB_CLAUDE_SESSION_ID` (_and_
also `JEB_CLAUDE_SESSION_ID_INITIAL` if they differ, multiple trailer in that
case, with initial coming first, but both with the same key of
`Agent-Session_ID`).

(This doesn't look at noedit files at all, but it's setting up the environment
for `Stop`.)

---

In `Stop` hook

We take our session ID from the first Agent-Session-ID in the HEAD commit, if
any are present, otherwise from `JEB_CLAUDE_SESSION_ID_INITIAL`, otherwise from
`JEB_CLAUDE_SESSION_ID`, otherwise (if none of those are set to a non-empty
string) we emit an error.

We scan back (first-parents only) to find out how many consecutive commits
starting at HEAD have the session ID as their `Agent-Session-ID` header (which
may be none). If a commit is marked as co-authored-by or committed-by or
authored-by a user with the email address noreply@anthropic.com, then if it has
no Agent-Session-ID trailers, we treat it as though it has one with the current
session ID. (i.e. if our tool doesn't mark the sessions correctly, but we can
still see it's from claude, we assume it's part of the current session).

We read the `.noedit` files from `JEB_CLAUDE_INITIAL_COMMIT` (i.e. we ignore any
changes to those files from this agent.) If there's no
`JEB_CLAUDE_INITIAL_COMMIT` in the env, then we look at the

Then we check for any agentlocked files

If we were already running in response to a stop hook, then we stop for good
here. If we weren't, then we ask the agent to continue. then we revert those
files to the state they had prior to those changes. `.noedit` files themselves
are, of course, the first thing we restore, so agents can't edit them out (or IN
-- agents cannot edit these, so agents cannot lock themselves out of files
either).
