#!/bin/bash
export TZ=UTC0

if [ -n "$CLAUDECODE" ]; then
    export GIT_COMMITTER_NAME="Claude Code"
    export GIT_COMMITTER_EMAIL="noreply@anthropic.com"

    if [ -n "$CLAUDE_CODE_REMOTE" ]; then
        export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME} (remote)"
    fi
elif [ -n "$GEMINI_CLI" ]; then
    export GIT_COMMITTER_NAME="Gemini CLI"
    export GIT_COMMITTER_EMAIL="noreply@google.com"
fi

git_save_commit() {
    git commit --allow-empty-message --no-edit --quiet || return

    default_message="$(git log -1 --format=%B)"
    tree="$(git write-tree)"
    tree_label="x$(echo "${tree:0:4}" | tr '[:lower:]' '[:upper:]')"
    git commit --amend -m "${tree_label}" -m "${default_message}"
}

git_save() {
    if command -v save >/dev/null 2>&1  || [ $# -gt 0 ]; then
        save "$@"
    else
        staged_tree="$(git write-tree)"
        git_save_commit
        staged_result="$?"

        git add "$(git rev-parse --show-toplevel)"
        unstaged_tree="$(git write-tree)"
        [ "${staged_tree}" != "${unstaged_tree}" ] && (echo; git_save_commit)
        unstaged_result="$?"

        if [ $staged_result -ne 0 ] && [ $unstaged_result -ne 0 ]; then
            return 1
        fi
    fi
}

git_save "$@"
