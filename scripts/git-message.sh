#!/bin/sh
set -eu

export TZ=UTC0

if [ -n "${CLAUDECODE:-}" ]; then
    export GIT_COMMITTER_NAME="Claude Code"
    export GIT_COMMITTER_EMAIL="noreply@anthropic.com"

    if [ -n "${CLAUDE_CODE_REMOTE:-}" ]; then
        export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME} (remote)"
    fi
elif [ -n "${GEMINI_CLI:-}" ]; then
    export GIT_COMMITTER_NAME="Gemini CLI"
    export GIT_COMMITTER_EMAIL="noreply@google.com"
fi

# Target commit: first argument, or most recent first-parent ancestor that is a merge
if [ -n "${1:-}" ]; then
    target="$1"
else
    # Walk first-parent ancestry until we find a merge commit
    # Fall back to HEAD~1 if no merge found before reaching root
    target="$(git rev-parse HEAD~1)"
    while ! git rev-parse --verify "$target^2" >/dev/null 2>&1; do
        if ! git rev-parse --verify "$target~1" >/dev/null 2>&1; then
            # Reached root without finding a merge, fall back to HEAD~1
            target="$(git rev-parse HEAD~1)"
            break
        fi
        target="$(git rev-parse "$target~1")"
    done
fi

# Resolve to full commit hash
target="$(git rev-parse "$target")"

# Verify target is an ancestor of HEAD
if ! git merge-base --is-ancestor "$target" HEAD; then
    echo "error: $target is not an ancestor of HEAD" >&2
    exit 1
fi

echo "reading stdin for message describing changes since $target:" >&2

# Get HEAD's tree and commit
tree="$(git rev-parse HEAD^{tree})"
head="$(git rev-parse HEAD)"

# Create new commit with:
# - Same tree as HEAD
# - First parent: target
# - Second parent: HEAD
# - Message from stdin (git commit-tree reads from stdin)
new_commit="$(git commit-tree "$tree" -p "$target" -p "$head")"

# Update HEAD to point to the new commit
git update-ref HEAD "$new_commit"

echo "created merge commit $new_commit" >&2
