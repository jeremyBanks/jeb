#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb stdin xml-to-jsonlines stdout <<'EOF'
<html lang="en">
  <body id="main">
    <div class="content">Hello</div>
  </body>
</html>
EOF
JEB



{"":"html","@index":0,"@tail":"","@text":"\n  ","lang":"en"}{"":"body","-":"html","-lang":"en","@index":0,"@tail":"","@text":"\n    ","id":"main"}{"":"div","-":"body","--":"html","--lang":"en","-id":"main","@index":0,"@tail":"\n","@text":"Hello","class":"content"}