#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



cat parse-html/index.html | jeb stdin first-64 parse-markup stdout
JEB



[
  {
    "": "!DOCTYPE",
    "@index": 0,
    "@tail": null,
    "@text": " html"
  },
  {
    "": "title",
    "@index": 0,
    "@tail": null,
    "@text": "Community Data Dump"
  },
  {
    "": "meta",
    "@index": 0,
    "@tail": null,
    "@text": "",
    "content": "width=685",
    "name": "viewport"
  }
]