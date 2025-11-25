#!/bin/bash
# shellcheck disable=all
set -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"
jeb() {
    echo "$(grep -m 1 -B 999 -A 1 "^jeb " < "$path")"$'\n\n\n' > "$path"
    cargo run --release --bin jeb -- "$@" >> "$path"
    exit
}



jeb help encode-jeb85 split-80 join-lines stdout



|# `jwNY3O3qak)1|N Entity Bag?kmd-&2|ust Encode Bytes?|kmd-&3|oined Escaped Bina
ry?|.kmd:vd|ibe-coded clanker slop with Semitranslucent Binary Encodings.|......
.....e^.}g|# LicensewEn=i7|opyright Jeremy Banks and contributors.|...A=S&u3qtez
3|ensed under either of:|w&Znah|- Apache License, Version 2.0 (<http://www.apach
e.org/licenses/LICENSE-2.0>)|................3n0w{a|IT license (<http://opensour
ce.org/licenses/MIT>)|........dgcz<1|t your optionzy#n61|## Contributix(v>@3rp$W
h|ess you explicitly state otherwise, any contribution intentionally submitted|.
...............3tai%h| inclusion in the work by you, as defined in the Apache-2.
0 license, shall be|...............wEq%<e|al licensed as above, without any addi
tional terms or conditions.|............e^.
