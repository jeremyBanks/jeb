#!/bin/bash
set -vexu

sudo chown "$USER:$USER" ./target

git config --global core.mergeoptions "--no-edit"
git config --global pull.default current
git config --global pull.rebase false
git config --global push.autoSetupRemote true
git config --global core.pager "less -F -X"
# shellcheck disable=SC2016,SC2101
git config --global alias.save '!
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
        git commit --allow-empty-message --no-edit >/dev/null 2>&1 || return

        default_message="$(git log -1 --format=%B)"
        tree="$(git write-tree)"
        tree_label="x$(echo "${tree:0:4}" | tr '[:lower:]' '[:upper:]')"
        git commit --amend -m "${tree_label}" -m "${default_message}"
    }

    git_save() {
        if command -v save >/dev/null 2>&1 || [ $# -gt 0 ]; then
            save "$@"
        else
            staged_tree="$(git write-tree)"
            git_save_commit
            staged_result="$?"

            git add "$(git rev-parse --show-toplevel)"
            unstaged_tree="$(git write-tree)"
            [ "${staged_tree}" != "${unstaged_tree}" ] && git_save_commit
            unstaged_result="$?"

            if [ $staged_result -ne 0 ] && [ $unstaged_result -ne 0 ]; then
                echo "nothing to commit, working tree clean" >&2
                return 1
            fi
        fi
    }

    git_save
'

rustup update
rustup show
rustup default nightly-2025-11-28

curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

cargo binstall --no-confirm --strategies crate-meta-data jj-cli

curl -fsSL https://claude.ai/install.sh | bash
