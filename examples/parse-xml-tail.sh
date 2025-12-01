#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



echo '<?xml version="1.0"?><root><a>text in a</a>tail after a<b>text in b</b>tail after b</root>' | jeb stdin parse-xml stdout
JEB



[
  {
    "": "?xml",
    "@index": 0,
    "@tail": null,
    "@text": " version=\"1.0\""
  },
  {
    "": "root",
    "@index": 0,
    "@tail": null,
    "@text": "tail after atail after b",
    "children": [
      {
        "": "a",
        "-": "root",
        "--": null,
        "@index": 0,
        "@tail": null,
        "@text": "text in a"
      },
      {
        "": "b",
        "-": "root",
        "--": null,
        "@index": 0,
        "@tail": null,
        "@text": "text in b"
      }
    ]
  }
]