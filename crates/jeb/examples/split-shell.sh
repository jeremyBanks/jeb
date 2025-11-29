#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



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

