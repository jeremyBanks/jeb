#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"
jeb() {
    echo "$(grep -m 1 -B 999 -A 8 "^jeb " < "$path")"$'\n\n\n' > "$path"
    cargo run --release --bin jeb -- "$@" >> "$path"
    exit
}



jeb '
    ./mirror.sh
    | encode-jeb85
    | split-80
    | join-lines
    | stdout
'



|#!/bin/bavrl8?4|# shellcheck disable=all|...3l{Wm4|pellchecker: disable=all|...
3uG^2e| -euo pipefail; path="$(realpath "$0")"; cd "$(dirname "$path")"|........
.....3tKcY|() {3lQDlf| echo "$(grep -m 1 -B 999 -A 8 "^jeb " < "$path")"$'\n\n\n
' > "$path"|.............a{]Cna|  cargo run --release --bin jeb -- "$@" >> "$pat
h"|.......xD^^a|   eCXImbEf{q*3tKcYasvHs1|   ./mirror.sB7+[62|   | encode-jeb85|
h3$DG1|  | split-80|3lQDl1| | join-linesA%:3j|  | stdouB-W%r3jmaE
