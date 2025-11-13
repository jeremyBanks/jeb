#!/bin/sh
set -vexu

git config --global push.autoSetupRemote current
git config --global pull.rebase false
