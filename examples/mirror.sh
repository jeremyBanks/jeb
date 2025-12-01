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



1|#!/bin/bvrl8?5|# shellcheck disable=all|...3l{Wm5|pellchecker: disable=all|...
3uHcd5|rce "$(dirname $0)/setup"|..a{[U:3tKcYasvHs2|   ./mirror.sB7+[63|   | enc
ode-jeb85|h3$DG2|  | split-80|3lQDl2| | join-linesA%:3j1|  | stdoB-W%r3q9-N3jmaE
