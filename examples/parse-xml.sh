#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



echo '<?xml version="1.0" encoding="UTF-8"?><root><item>First</item><item>Second</item><item>Third</item></root>' | jeb stdin parse-xml stdout
JEB



[
  {
    "": "?xml",
    "-": null,
    "@index": 0,
    "@tail": null,
    "@text": " version=\"1.0\" encoding=\"UTF-8\""
  },
  {
    "": "item",
    "-": "root",
    "--": null,
    "@index": 0,
    "@tail": null,
    "@text": "First"
  },
  {
    "": "item",
    "-": "root",
    "--": null,
    "@index": 1,
    "@tail": null,
    "@text": "Second"
  },
  {
    "": "item",
    "-": "root",
    "--": null,
    "@index": 2,
    "@tail": null,
    "@text": "Third"
  },
  {
    "": "root",
    "-": null,
    "@index": 1,
    "@tail": "\n",
    "@text": null
  }
]