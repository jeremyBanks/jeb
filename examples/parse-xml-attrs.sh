#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



echo '<?xml version="1.0"?><book id="123"><title>Example</title><author name="Jane"><email>jane@example.com</email></author></book>' | jeb stdin parse-xml stdout
JEB



{
  "": "book",
  "@index": 0,
  "@tail": null,
  "@text": null,
  "children": [
    {
      "": "title",
      "-": "book",
      "--": null,
      "-id": "123",
      "@index": 0,
      "@tail": null,
      "@text": "Example"
    },
    {
      "": "author",
      "-": "book",
      "--": null,
      "-id": "123",
      "@index": 0,
      "@tail": null,
      "@text": null,
      "children": [
        {
          "": "email",
          "-": "author",
          "--": "book",
          "---": null,
          "--id": "123",
          "-name": "Jane",
          "@index": 0,
          "@tail": null,
          "@text": "jane@example.com"
        }
      ],
      "name": "Jane"
    }
  ],
  "id": "123"
}