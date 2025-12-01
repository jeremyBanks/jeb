#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



echo '<root><item>First</item><item>Second</item><item>Third</item></root>' | jeb stdin parse-xml stdout
JEB



{
  "": "root",
  "@index": 0,
  "@tail": null,
  "@text": null,
  "children": [
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
      "@index": 0,
      "@tail": null,
      "@text": "Second"
    },
    {
      "": "item",
      "-": "root",
      "--": null,
      "@index": 0,
      "@tail": null,
      "@text": "Third"
    }
  ]
}