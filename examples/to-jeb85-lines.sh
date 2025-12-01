#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb ./to-jeb85-lines.sh to-jeb85-lines stdout
JEB



1|#!/bin/bvrl8?5|# shellcheck disable=all|...3l{Wm5|pellchecker: disable=all|...
3uG^2i| -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_fla
gs=(|...............c<?y+i|echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\
n\n\n' > "$path"; jeb() {|..............aBsD9h|argo run "${cargo_flags[@]}" --bi
n jeb -- "$@" >> "$path"; exit; }; set -x|.............eQdQf3jpJV9|b ./to-jeb85-
lines.sh to-jeb85-lines stdout|....z/{den<#5N3jma
