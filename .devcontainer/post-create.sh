#!/bin/bash
set -vexu

git config --global core.mergeoptions "--no-edit"
git config --global pull.default current
git config --global pull.rebase false
git config --global push.autoSetupRemote true

rustup update
rustup show
