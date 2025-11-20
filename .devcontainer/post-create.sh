#!/bin/sh
set -vexu

git config --global push.autoSetupRemote true
git config --global pull.default current
git config --global pull.rebase false

curl -fsSL https://claude.ai/install.sh | bash

rustup update
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly

cargo install --path crates/jeb
