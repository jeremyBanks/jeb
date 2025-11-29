#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"
jeb() {
    echo "$(grep -m 1 -B 999 -A 1 "^JEB" < "$path")"$'\n\n\n' > "$path"
    cargo run --release --bin jeb -- "$@" >> "$path"
    exit
}



jeb stdin split-shell join-lines stdout <<'JEB'
    echo '\hello world' \"\f\o\o bar\" baz\ qux th"i"s and\
        that
JEB



echo
\hello world
"foo
bar"
baz qux
this
and
that

