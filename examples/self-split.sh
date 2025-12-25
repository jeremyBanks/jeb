#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



cargo_flags=(--release)

jeb self split-64 encode-jeb85 find-'|' first-32 join-lines stdout
JEB



}#M&#00000l&Wh!00000l&Wh!000001onA4000004|/lib64/ld-linux-x86-64.so.2|z.r^B1onA4
5c8Xg0rr91m]04:000000@@r30SSi2000001onA46Awak0@@r3m]04:6NRm$ltVzRsp@{hF=zt)||_c6
1|ibc_start_maix(i]T1|_gmon_start__uJsC84|TM_deregisterTMCloneTable|..wDl7e|TM_r
2|egisterTMCloneTablvR/PQ1|__cxa_finalizDsW7-1|Unwind_ResumewDlNU|mmovwDlNU|mcpy
0aGCPz#:-^|mset0af8P|ls_get_adwmPYX|memczeYF+|lose0aY[M1|iterate_phdr|0bV<@zddw-
|syscvqG-W3|pthread_attr_destroy|..0chTVwb]$z1|errno_locatioz/bL!|stathz+zv|Unwi
|nd_GetIP|0b2>+Bzkm=3|_Unwind_GetTextRelBase|B7CtU3|Unwind_GetDataRelBase|.wDlvO
|tenv0crm5y?mF:|mallz^@=^1|osix_memalignzuao?wN$<Z|eallz^@=>|rite0aPCNy&r!-|poll
0a].UBzH@3|riteB]VO$|gnal0crBb|conf0c03[|read_joinzuaf.|ock_gettix(dzY||_Unwind_
|BacktracewDlT/|en640bMZ/|ek640aZj>0axtPA=.q2|thread_sewO+@U||pthread_getattr_
zG2O{3|thread_attr_getstack|..0cq<!|actiz/bL]|ause0b2>+v%qC-|stathz+zJ|map6gYRMc
|alpaBz7U{|eadlx(mX^|statCMqw/|tranwn=q+4|__cxa_thread_atexit_impl|...0c03[|read
|_key_creavru5X2|pthread_key_deleteBy+C>2|thread_setspecificx>4cOCXIm1||getauxva
yYFJ<|galtstackyxer}3|hread_attr_getguardsizex)Ks<|mprotect|0c03[|read_detavpJ%J
2|__xpg_strerror_r|.0ae-z5|wind_GetLanguageSpecificData|....0ae-z||wind_GetIPIn
w]vVR3|Unwind_GetRegionStart|.Bo2VF|nwind_SetBvPje1|_Unwind_SetIPpYK<5||nwind_Ra
1|iseException|0ae-z3|wind_DeleteException|..0bMvU|gcc_s.so.e?)t@|CC_3e?^n}|CC_3
e&9F$|CC_4.2.0|0bMvU|c.soe&AY1|LIBC_2.2.e&rS0|LIBC_2.3|07PZO|BC_2.3.4|07PZO|BC_2
e?]f.|GLIBC_2.1f/x24|LIBC_2.17hVMuw|IBC_2.18|07PZO|BC_2e&2o:|GLIBC_2.2gb]n7|LIBC
|_2.3fAsQp|IBC_2.32|07PZO|BC_2e&bo-|GLIBC_2.3gC/8F2|d-linux-x86-64.so.z.r^B00000
0d$pL0d$pZ}VcYo0s[e>0mH%p00000%c<q}1s1mb0E>7H000000000000000||~/runner/.cargo/
||registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.4
3|.13/src/util/pool.rs|..055wn||stc/c86564c412a5949088a53b665d8b9a47ec61
5|0a39/library/alloc/src/str.rs|...A@W>l||ustc/c86564c412a5949088a53b665d8
7|b9a47ec610a39/library/std/src/io/mod.rs|...e](1kZYtN[0sw/W||rustc/c86564c412
c|a5949088a53b665d8b9a47ec610a39/library/alloc/src/vec/mod.rs|........e](1k|~/ru
||nner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-
7|automata-0.4.13/src/nfa/thompson/nfa.rs|...e](1k2Y>9d|ide:aIX0+0dDOw||unner/.c
||argo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/
2|src/sync/notify.rsA=Rj%||ibrary/std/src/../../backtrace/src/symbolize
1|/gimli/elf.rsA@V}(3|emory allocation of |..ZZ.F@|ytes faily?mb:0kFT%aIW#a|/run
||ner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-a
