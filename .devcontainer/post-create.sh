#!/bin/bash
set -vexu

git config --global core.mergeoptions "--no-edit"
git config --global pull.default current
git config --global pull.rebase false
git config --global push.autoSetupRemote true
git config --global core.pager "less -F -X"

rustup update
rustup show
rustup default nightly-2025-11-28

curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

cargo binstall --no-confirm --strategies crate-meta-data jj-cli

curl -fsSL https://claude.ai/install.sh | bash
