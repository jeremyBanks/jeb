#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"
jeb() {
    echo "$(grep -m 1 -B 999 -A 1 "^jeb " < "$path")"$'\n\n\n' > "$path"
    cargo run --release --bin jeb -- "$@" >> "$path"
    exit
}



jeb help collapse encode-jeb85 split-80 join-lines stdout



||# `jeb` JSON Entity Bag? Just Encode Bytes? Joined Escaped Binary? vibe-coded 
clanker slop with Semitranslucent Binary Encodings. ## License Copyright Jeremy 
Banks and contributors. Licensed under either of: - Apache License, Version 2.0 
(<http://www.apache.org/licenses/LICENSE-2.0>) - MIT license (<http://opensource
.org/licenses/MIT>) at your option. ## Contribution Unless you explicitly state 
otherwise, any contribution intentionally submitted for inclusion in the work by
 you, as defined in the Apache-2.0 license, shall be dual licensed as above, wit
hout any additional terms or conditions.
