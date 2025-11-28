#!/bin/bash
set -vexu

git config --global push.autoSetupRemote true
git config --global pull.default current
git config --global pull.rebase false

rustup update
rustup show
