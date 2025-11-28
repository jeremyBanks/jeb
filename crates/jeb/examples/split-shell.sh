#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"
jeb() {
    echo "$(grep -m 1 -B 999 -A 1 "^jeb " < "$path")"$'\n\n\n' > "$path"
    echo "echo 'hello world' \"foo bar\" baz\\ qux" | cargo run --release --bin jeb -- "$@" >> "$path"
    exit
}



jeb stdin split-shell join-lines stdout



echo
hello world
foo bar
baz qux

