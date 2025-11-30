#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb help encode-jeb85 split-80 join-lines stdout
JEB



g|Incomplete and incorrect, vague ideas and hallucinations. At least for now.|..
..........z*9/v3n0?Y3jn12|`jebu<Wlk1|SON Entity Bavp}!i3qbCZ1|t Encode BytewPH6y
3qbkJ2|ned Escaped BinaryA^o>R3r7IM5|itranslucent Binary Encodings?|..B3y0s|## L
icensB7CW=8|Copyright Jeremy Banks and contributors.|.......3joEv3|censed under 
either of:z!q3q3n0w^g|pache License, Version 2.0 (<http://www.apache.org/license
s/LICENSE-2.0>)|..............dgdTHa|MIT license (<http://opensource.org/license
s/MIT>)|.......j$c*C1|at your optioz/dfm3l{=r1|Contribution|3jo^Jh|less you expl
icitly state otherwise, any contribution intentionally submitted|...............
wc#{!h|r inclusion in the work by you, as defined in the Apache-2.0 license, sha
ll be|..............vR2)Se|ual licensed as above, without any additional terms o
r conditions.|...........B1N!
