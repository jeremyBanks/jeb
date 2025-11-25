#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"
jeb() {
    echo "$(grep -m 1 -B 999 -A 1 "^jeb " < "$path")"$'\n\n\n' > "$path"
    cargo run --release --bin jeb -- "$@" >> "$path"
    exit
}



jeb help split-64 encode-jeb85 join-lines stdout



|# `jwNY3O3qak)1|N Entity Bag?kmd-&2|ust Encode Bytes?|kmd-&||oined Escaped Bi
|narykmd:v||ibe-coded clanker slop with Semitranslucent Binary Encod
|ingse^.}g|# LicensewEn=i7|opyright Jeremy Banks and contributors.|...A=S&u3qtez
3|ensed under either of:|w&Zna||- Apache License, Version 2.0 (<http://w
7|ww.apache.org/licenses/LICENSE-2.0>)|......3n0w{||IT license (<http://open
4|source.org/licenses/MIT>)|..dgcz<1|t your optionzy#n61|## Contributix(v>@3rp$W
||ess you explicitly state otherwise, any contribution intentional
1|ly submitted|3tai%|| inclusion in the work by you, as defined in the
5| Apache-2.0 license, shall be|...wEq%<||al licensed as above, without an
6|y additional terms or conditions.|....e^.
