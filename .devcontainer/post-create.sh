#!/bin/bash
set -vexu

sudo chown "$USER:$USER" ./target

git config --global core.mergeoptions "--no-edit"
git config --global pull.default current
git config --global pull.rebase false
git config --global push.autoSetupRemote true
git config --global core.pager "less -F -X"
# shellcheck disable=SC2016
git config --global alias.save '!./run save'

rustup update
rustup show
rustup default nightly-2025-11-28

curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

binstall=(
    cargo-nextest
    tracey
    jj-cli
)
cargo binstall --secure --no-confirm --strategies crate-meta-data "${binstall[@]}"

curl -fsSL https://claude.ai/install.sh | bash
