#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname "$0")/setup"

jeb stdin split-whitespace filter join decode-z85 encode-binary split-32 join-lines stdout  <<'JEB'
    00000
    00001
    00002
    00004
    00008
    0000g
    0000w
    0000:
    0001H
    00031
    00062
    000c4
    000o8
    000Mg
    001bw
    002m:
    004JH
    00961
    00ic2
    00Ao4
    00&M8
    01Ybg
    03zmw
    06*I:
    0dU4H
    0rr91
    0SSi2
    1onA4
    2MK&8
    5c8Xg
    aohxw
    kMy=:
    Fb/MH
JEB

