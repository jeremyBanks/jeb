#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb help split-64 encode-jeb85 join-lines stdout
JEB



4|incomplete and incorrect|...3jnvpeEz?f| `jevQGJI2|JSON Entity Bag?|.3joyF|st E
1|ncode Bytes?|3joyz3|ined Escaped Binary?|..3joZy||mitranslucent Binary Enc
|odinxlcoA3l{=r|LicezGxv@3pw-J7|yright Jeremy Banks and contributors.|.....e^.}V
4|icensed under either of:|...3jnvc||Apache License, Version 2.0 (<http:/
7|/www.apache.org/licenses/LICENSE-2.0>)|....j$c*<|| MIT license (<http://op
4|ensource.org/licenses/MIT>)|r6z-r3sO301|your option.|3jn151| Contributionzvc.J
||nless you explicitly state otherwise, any contribution intention
1|ally submittewN(cW||or inclusion in the work by you, as defined in t
5|he Apache-2.0 license, shall be|.ayX)?||dual licensed as above, without 
6|any additional terms or conditions.|..zGvAq
