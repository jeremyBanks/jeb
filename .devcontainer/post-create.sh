#!/bin/bash
set -vexu

git config --global core.mergeoptions "--no-edit"
git config --global pull.default current
git config --global pull.rebase false
git config --global push.autoSetupRemote true
git config --global core.pager "less -F -X"
# shellcheck disable=SC2016,SC2101
git config --global alias.save '!
    git_commit_tree() {
        tree="$(git write-tree)"
        if git commit --allow-empty-message --no-edit >/dev/null 2>&1; then
            default_message="$(git log -1 --format=%B)"
            tree_label="x$(echo "${tree:0:4}" | tr '[:lower:]' '[:upper:]')"
            git commit --amend -m "${tree_label}" -m "${default_message}"
        fi
        echo "${tree}"
    }

    git_save() {
        if command -v save >/dev/null 2>&1 || [ $# -gt 0 ]; then
            save "$@"
        else
            staged_tree="$(git_commit_tree)"
            staged_result="$?"

            git add "$(git rev-parse --show-toplevel)"
            unstaged_tree="$(git write-tree)"
            [ "${staged_tree}" != "${unstaged_tree}" ] && git_commit_tree
            unstaged_result="$?"

            [ $staged_result -eq 0 ] || return $unstaged_result
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
