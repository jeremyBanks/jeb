#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



echo '<html><body><h1>Hello</h1><p>World</p></body></html>' | jeb stdin parse-html stdout
JEB



{
  "": "html",
  "@index": 0,
  "@tail": null,
  "@text": null,
  "children": [
    {
      "": "body",
      "-": "html",
      "--": null,
      "@index": 0,
      "@tail": null,
      "@text": null,
      "children": [
        {
          "": "h1",
          "-": "body",
          "--": "html",
          "---": null,
          "@index": 0,
          "@tail": null,
          "@text": "Hello"
        },
        {
          "": "p",
          "-": "body",
          "--": "html",
          "---": null,
          "@index": 0,
          "@tail": null,
          "@text": "World"
        }
      ]
    }
  ]
}