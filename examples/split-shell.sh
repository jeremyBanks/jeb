#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname "$0")/setup"

jeb stdin split-shell join-lines stdout <<'JEB'
    echo '\hello world' \"\f\o\o bar\" baz\ qux th"i"s and\
        that\
JEB

echo
\hello world
"foo
bar"
baz qux
this
and
that
