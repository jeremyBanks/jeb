#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname "$0")/setup"

cargo_flags=(--release)
jeb self split-64 encode-jeb85 find-\| find-.rs first-256 join-lines stdout
