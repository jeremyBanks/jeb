#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb ./to-jeb85-lines.sh to-jeb85-lines stdout
JEB



|#!/bin/bavrl8?4|# shellcheck disable=all|...3l{Wm4|pellchecker: disable=all|...
3uG^2h| -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_fla
gs=(|...............c<?y+h|echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\
n\n\n' > "$path"; jeb() {|..............aBsD9g|argo run "${cargo_flags[@]}" --bi
n jeb -- "$@" >> "$path"; exit; }; set -x|.............eQdQf3jpJV8|b ./to-jeb85-
lines.sh to-jeb85-lines stdout|....z/{den<#5N3jma
