#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb help encode-jeb85 split-80 join-lines stdout
JEB



4|incomplete and incorrect|...3jnvpeEz?f| `jevQGJI2|JSON Entity Bag?|.3joyF2|st 
Encode Bytes?|.3joyz3|ined Escaped Binary?|..3joZy5|mitranslucent Binary Encodin
gs?|.xlcoA3l{=r|LicezGxv@3pw-J7|yright Jeremy Banks and contributors.|.....e^.}V
4|icensed under either of:|...3jnvcg|Apache License, Version 2.0 (<http://www.ap
ache.org/licenses/LICENSE-2.0>)|.............j$c*<a| MIT license (<http://openso
urce.org/licenses/MIT>)|......r6z-r3sO301|your option.|3jn151| Contributionzvc.J
h|nless you explicitly state otherwise, any contribution intentionally submitted
|..............wN(cWh|or inclusion in the work by you, as defined in the Apache-
2.0 license, shall be|.............ayX)?e|dual licensed as above, without any ad
ditional terms or conditions.|..........zGvAq
