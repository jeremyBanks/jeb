#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb help encode-jeb85 split-80 join-lines stdout
JEB



|# `jwNY3O3qak)1|N Entity Bag?kmd-&2|ust Encode Bytes?|kmd-&3|oined Escaped Bina
ry?|.kmd-83jp@<d|be-coded clanker slop with Semitranslucent Binary Encodings.|..
..........3jmaE|## LicensB7CW=8|Copyright Jeremy Banks and contributors.|.......
3joEv3|censed under either of:z!q3q3n0w^g|pache License, Version 2.0 (<http://ww
w.apache.org/licenses/LICENSE-2.0>)|..............dgdTHa|MIT license (<http://op
ensource.org/licenses/MIT>)|.......j$c*C1|at your optioz/dfm3l{=r1|Contribution|
3jo^Jh|less you explicitly state otherwise, any contribution intentionally submi
tted|...............wc#{!h|r inclusion in the work by you, as defined in the Apa
che-2.0 license, shall be|..............vR2)Se|ual licensed as above, without an
y additional terms or conditions.|...........B1N!
