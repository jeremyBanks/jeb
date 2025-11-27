#!/bin/sh
set -vexu

git config --global push.autoSetupRemote true
git config --global pull.default current
git config --global pull.rebase false

curl -fsSL https://claude.ai/install.sh | bash

rustup update
rustup target add aarch64-unknown-linux-gnu
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
rustup default nightly

history -s "cargo fix --allow-dirty; cargo clippy --fix --allow-dirty; cargo fmt; crates/jeb/examples/all.sh"
history -s "git commit --allow-empty-message --no-edit; git add ."
