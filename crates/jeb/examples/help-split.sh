#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb help split-64 encode-jeb85 join-lines stdout
JEB



|# `jwNY3O3qak)1|N Entity Bag?kmd-&2|ust Encode Bytes?|kmd-&||oined Escaped Bi
|narykmd-83jp@<||be-coded clanker slop with Semitranslucent Binary En
|codings.|3jmaE|## LicensB7CW=||Copyright Jeremy Banks and contributors.
3joEv3|censed under either of:z!q3q3n0w^||pache License, Version 2.0 (<htt
8|p://www.apache.org/licenses/LICENSE-2.0>)|......dgdTH||MIT license (<http:/
5|/opensource.org/licenses/MIT>)|..j$c*C1|at your optioz/dfm3l{=r||Contribution
3jo^J||less you explicitly state otherwise, any contribution intent
2|ionally submitted|wc#{!||r inclusion in the work by you, as defined i
6|n the Apache-2.0 license, shall be|...vR2)S||ual licensed as above, witho
7|ut any additional terms or conditions.|....B1N!
