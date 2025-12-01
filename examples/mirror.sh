#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname $0)/setup"



jeb '
    ./mirror.sh
    | encode-jeb85
    | split-80
    | join-lines
    | stdout
'
JEB



|#!/bin/bavrl8?4|# shellcheck disable=all|...3l{Wm4|pellchecker: disable=all|...
3uHcd4|rce "$(dirname $0)/setup"|..a{[U:3tKcYasvHs1|   ./mirror.sB7+[62|   | enc
ode-jeb85|h3$DG1|  | split-80|3lQDl1| | join-linesA%:3j|  | stdouB-W%r3q9-N3jmaE
