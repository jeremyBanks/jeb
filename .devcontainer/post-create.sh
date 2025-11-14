#!/bin/sh
set -vexu

git config --global push.autoSetupRemote current
git config --global pull.rebase false

rustup update
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
