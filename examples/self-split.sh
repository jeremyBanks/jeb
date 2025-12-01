#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname "$0")/setup"

cargo_flags=(--release)
jeb self split-64 encode-jeb85 find-\| find-.rs first-256 join-lines stdout

6|-0.3.22/src/registry/sharded.rs|.e](1k0|k[tracing-subscriber] Unable to 
b|o-1949cf8c6b5b557f/color-spantrace-0.3.0/src/lib.rs|......e](1k0v$}=3tBc<|orin
8|7f/tracing-core-0.1.35/src/callsite.rs|....A=Rkf0|/.cargo/registry/src/ind
a|8c6b5b557f/regex-syntax-0.8.8/src/ast/parse.rs|......A=Rkf0|/.cargo/registry
4|ry/alloc/src/vec/mod.rse](1k0|library/std/src/../../backtrace/src/symb
3|olize/gimli/elf.rsA=Ri}4|memory allocation of |.aIXEf2|bytes failed|3ikJd|.rus
a|ustlib/src/rust/library/core/src/str/pattern.rs|.....e](1k7J:Xi0|ed to clone 
1|imize.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/
b|regex-automata-0.4.13/src/nfa/thompson/compiler.rs|.......A=Rjj0|first captur
0aQ4T3|tes/jeb/src/z85.rsA=Rkf0|/.rustup/toolchains/nightly-2025-11-28-x
1|ead/mod.A=RiYf2km57|library/std/src/sys/pal/unix/os.rs|...A=Rjl0|rust/deps/gi
6|mli-0.32.3/src/read/abbrev.rs|...A@V+W2|ssertion `lefw]]fh5fM1c0|ght` failed:
d|io-1949cf8c6b5b557f/backtrace-0.3.76/src/symbolize/gimli.rs|........e](1k6>8ia
6|1.1.4/src/nfa/noncontiguous.rs|..A=Rkf0|/.cargo/registry/src/index.crate
e|s.io-1949cf8c6b5b557f/regex-syntax-0.8.8/src/hir/interval.rs|............0dDNN
5|ta-0.4.13/src/util/pool.rs|.A=Ri>2|prelude errorAYLtU0dDNN0|cargo/registry/s
3|c/fmt/fmt_layer.rsA=RjD0|Thread count overflowed the configured max c
a|9cf8c6b5b557f/rustc-demangle-0.1.26/src/v0.rs|.......A@V}?0|nvalid 'id2' sta
9|ex-automata-0.4.13/src/nfa/thompson/map.rs|.....A=Rkf0|/.cargo/registry/src
1|/look.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/
8|regex-automata-0.4.13/src/dfa/search.rs|...e](1k0|~/.cargo/registry/src/in
|te.rA@W>l9|ust/deps/rustc-demangle-0.1.26/src/v0.rs|.......0bMvU0|rary/alloc/s
b|racing-subscriber-0.3.22/src/filter/env/field.rs|.........0dDNN0|cargo/regist
|b.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/back
8|trace-0.3.76/src/backtrace/libunwind.rs|...e](1k0|~/.rustup/toolchains/nig
a|brary/alloc/src/collections/btree/navigate.rs|.......A@-#Ri>$.b0|pattern leng
b|8c6b5b557f/regex-automata-0.4.13/src/util/search.rs|......e](1k6kg*10|stack of
c|9cf8c6b5b557f/aho-corasick-1.1.4/src/util/remapper.rs|.........A@WB@0|xpected 
3|c/src/vec/mod.rs|.00)dL|apsewiEZk|; whwO%VD00%0O|hreawfrnn11bitZZ.!o0|panicked
9|tlib/src/rust/library/core/src/cell/once.rs|....e](1k8:)dY0| exceeded size l
3|mpson/backtrack.rsA=Rkf0|/.cargo/registry/src/index.crates.io-1949cf8
0|c6b5b557f/regex-automata-0.4.13/src/nfa/thompson/literal_trie.rs
e|.io-1949cf8c6b5b557f/aho-corasick-1.1.4/src/util/prefilter.rs|...........A@ZUx
7|0/src/runtime/context/runtime.rs|.....0bMvU0|rary/std/src/io/buffered/lin
b|nu/lib/rustlib/src/rust/library/alloc/src/string.rs|......e](1k0xO0@0|~/.cargo
4|src/symbolize/mod.rs|..048Pa8|mber of DFA states exceeds limit of |......ZYj*M
c|8c6b5b557f/regex-automata-0.4.13/src/util/primitives.rs|.......e](1k6kquc|alid
1|error.rs01-^Y2|t codepoint Urv+<i0| which occurs before last codepoint 
6|-1.48.0/src/util/wake_list.rs|...A@WU30|ailed to allocate an alternative
a|f8c6b5b557f/tracing-core-0.1.35/src/field.rs|........0266:0|valid pattern ID
a|automata-0.4.13/src/util/determinize/state.rs|.......A@ZUx0|.cargo/registry/
2|ed/pattern.rsA@VCR|rroriV#ir0|1exceeded the maximum number of capturin
3|rc/ffi/os_str.rs|.055wn9|st/deps/addr2line-0.25.1/src/function.rs|.......0bMvU
7|rary/std/src/sync/reentrant_lock.rs|..e](1k0dDNN0|cargo/registry/src/index
3|filter/byteset.rs|A@VO!1|refilteriV#is3ih-10|ustc/c86564c412a5949088a53b6
7|src/collections/btree/map/entry.rs|...A=Rjl0|rustc/c86564c412a5949088a53b
9|665d8b9a47ec610a39/library/core/src/time.rs|....e](1k0|library/std/src/../.
9|./backtrace/src/symbolize/gimli/stash.rs|.......04IM80|tension cannot conta
4|in path separators: |..ZYng+5|brary/core/src/fmt/num.rs|..A@YY<0|ates/jeb/src
3|-0.2.0/src/lib.rs|A@ZUx0|.cargo/registry/src/index.crates.io-1949cf8c
a|6b5b557f/lazy_static-1.5.0/src/inline_lazy.rs|.......A@ZUx0|.cargo/registry/
1|egacy.rs0kGr(4|exceeds capacity of |..ZZ}Sj2|hen insertingxcs$&0|~/.cargo/reg
4|/src/hybrid/search.rs|.A@ZUx0|.cargo/registry/src/index.crates.io-1949
9|cf8c6b5b557f/aho-corasick-1.1.4/src/dfa.rs|.....A=Ri:|Spand8k)naIX1b0dDNN|carg
5|/runtime/scheduler/defer.rs|e](1k0|library/std/src/../../backtrace/src/
4|symbolize/gimli/lru.rs|A=Rj%8|ibrary/core/src/num/dec2flt/parse.rs|......048h]
9|f8c6b5b557f/sharded-slab-0.1.7/src/shard.rs|....e](1k0|~/.cargo/registry/sr
8|7f/thread_local-1.1.9/src/thread_id.rs|....A=Rj30|accelerator already cont
8|7f/aho-corasick-1.1.4/src/automaton.rs|....A=Rjf0|tried to unwrap expr fro
3|m HirFrame, got: |aIW$g8|rust/deps/gimli-0.32.3/src/read/line.rs|...e](1k|libr
b|ary/std/src/../../backtrace/src/symbolize/mod.rs|.........01imY1|te indexaIX<q
e](1k7|library/alloc/src/raw_vec/mod.rs|.....01%r-3|moval index (is |.Z.Ovw|shou
3|ld be < len (is |.ZYt>:7|library/core/src/fmt/builders.rs|.....03kY(0|ror: unr
e|c/index.crates.io-1949cf8c6b5b557f/addr2line-0.25.1/src/unit.rs|.........e](1k
a|7f/memchr-2.7.6/src/arch/generic/packedpair.rs|......A=Rkf0|/.rustup/toolcha
8|rust/library/alloc/src/raw_vec/mod.rs|.....A@XEt0|nternal error: entered u
4|c/symbolize/gimli.rs|..09ADE0|racing-subscriber] Unable to write an ev
a|gex-automata-0.4.13/src/util/determinize/mod.rs|.....e](1k0|~/.cargo/registr
a|b557f/aho-corasick-1.1.4/src/util/alphabet.rs|.......A@W!l0|ried to unwrap a
|c.rs0kF%+1|(os erroAV&c#dfaf#0|brary/core/src/num/flt2dec/strategy/gris
|u.rs07yal0|racing-subscriber] Unable to format the following event,
4|/stable/quicksort.rs|..00TM&|ith ZYn*p0|.cargo/registry/src/index.crates
d|.io-1949cf8c6b5b557f/miniz_oxide-0.8.9/src/inflate/core.rs|.........A=Rk@0wh7&
8|ex-automata-0.4.13/src/util/alphabet.rs|...e](1k0|~/.cargo/registry/src/in
2|dy/generic.rsA@ZUx0|.cargo/registry/src/index.crates.io-1949cf8c6b5b
a|557f/regex-syntax-0.8.8/src/hir/translate.rs|........0dDNN0|cargo/registry/s
7|f/tokio-1.48.0/src/runtime/park.rs|...A=Rj%0|ibrary/std/src/sys/thread/un
|ix.rA@Z2<a|brary/std/src/sys/pal/unix/stack_overflow.rs|........02xS(0|ice inde
2|x starts at |ZZRz}1|ut ends vrrTc0bMvU5|rary/core/src/str/lossy.rs|.A=Rkf|/.ru
a|rustlib/src/rust/library/std/src/sync/once.rs|.......A@ZUx0|.cargo/registry/
1|/slot.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/
d|tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs|...........04IY2|tern
1|i/lru.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/
8|regex-automata-0.4.13/src/util/utf8.rs|....A=Rkf0|/.cargo/registry/src/ind
7|kio-1.48.0/src/runtime/handle.rs|.....0bMvU0|rary/std/src/sys/pal/unix/mo
|d.rs0bMvU5|rary/core/src/panicking.rs|.A=Rkf0|/.rustup/toolchains/nightly-
3|/alloc/src/str.rs|A@ZUx0|.cargo/registry/src/index.crates.io-1949cf8c
b|6b5b557f/regex-automata-0.4.13/src/meta/wrappers.rs|......e](1k3$AZ+0|ture(pid
e|io-1949cf8c6b5b557f/regex-automata-0.4.13/src/hybrid/regex.rs|...........A@Wa)
c|s.io-1949cf8c6b5b557f/tokio-1.48.0/src/sync/notify.rs|.........A@V>=0|ailed pr
1|inting tzY(*%iV#ir7|library/std/src/sync/lazy_lock.rs|....A@Z2<0|brary/std/sr
4|c/sys/random/linux.rs|.A@Z2<4|brary/std/src/time.rs|.A@Ws%0|ndex out of boun
3|rc/dispatcher.rs|.0dDNN0|rustup/toolchains/nightly-2025-11-28-x86_64-
0|unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/sync.rs
7|tomata-0.4.13/src/util/escape.rs|.....01ATW1|valid spvqWj67:9Be0|r haystack o
c|f/regex-automata-0.4.13/src/util/prefilter/memchr.rs|..........0dDNN0|cargo/re
3|rc/packed/api.rs|.00)HX| linwGX+{1| (columnaIXHp2| through linewGX+{0| (column
5|x-syntax-0.8.8/src/debug.rs|e](1k4|crates/jeb/src/jeb85.rse](1k0|/rustc/c8656
3|rc/num/wrapping.rsA=Rj%4|ibrary/std/src/rt.rs|..0bMvU0|rary/std/src/thread/
1|current.A=Rj%8|ibrary/core/src/unicode/printable.rs|......04]{m0|rates/jeb/sr
2|btree/node.rsA@W4*4|xpected char at offset wPP6g0dDNN0|cargo/registry/src/i
0|ndex.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/sync/oneshot.rs
0bMvU5|rary/std/src/panicking.rs|..A@ZUx0|.cargo/registry/src/index.crates
b|.io-1949cf8c6b5b557f/sharded-slab-0.1.7/src/tid.rs|.......A=Rkf0|/.cargo/regi
d|-1949cf8c6b5b557f/regex-automata-0.4.13/src/dfa/regex.rs|...........00jn>ZYt>:
1|osition(z-P2R1Up^jiV#iw|, c:aIX1b0bMvU4|rary/std/src/path.rs|..055wn0|st/deps/
6|gimli-0.32.3/src/read/index.rs|..A=Rjl0|rust/deps/miniz_oxide-0.8.9/src/
2|inflate/core.e](1k6|library/core/src/num/bignum.rs|..A=Rj%0|ibrary/core/src/
5|num/dec2flt/decimal_seq.rs|.A=Rkf0|/.cargo/registry/src/index.crates.io
|y.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/shar
6|ded-slab-0.1.7/src/page/mod.rs|..A=Rk@0XtFfZZ8Ml|ourcwPLS*iV#ir1V5b1h9nON|~/.c
6|tomata-0.4.13/src/dfa/accel.rs|..A=RjM0|internal error: entered unreacha
1|earch.rs04J8h9|ied to unwrap group from HirFrame, got: |.......ZYl3f0|ried to 
a|/tokio-1.48.0/src/runtime/blocking/shutdown.rs|......A=Rj%0|ibrary/std/src/s
8|ary/alloc/src/collections/btree/node.rs|...e](1k0|/rustc/c86564c412a594908
c|8a53b665d8b9a47ec610a39/library/core/src/str/pattern.rs|.......e](1k0|5fatal r
7|ary/std/src/sys/pal/unix/time.rs|.....02ouZ3|mory allocation ofz!pcc4iPb9|tes 
9|c6b5b557f/tracing-core-0.1.35/src/span.rs|......A@WN#0|eterminization excee
e|8c6b5b557f/regex-automata-0.4.13/src/nfa/thompson/builder.rs|............07ps9
6|-1.1.4/src/nfa/contiguous.rs|....0dDNN0|cargo/registry/src/index.crates.
e|io-1949cf8c6b5b557f/aho-corasick-1.1.4/src/packed/rabinkarp.rs|..........A=Ri-
c|tracing-subscriber-0.3.22/src/filter/env/directive.rs|.........A@ZUx0|.cargo/r
5|ch/all/packedpair/mod.rs|...0dDNN0|cargo/registry/src/index.crates.io-1
e|949cf8c6b5b557f/regex-automata-0.4.13/src/util/sparse_set.rs|............0dDNN
5|-0.8.8/src/hir/literal.rs|..A@ZUx0|.cargo/registry/src/index.crates.io-
c|1949cf8c6b5b557f/aho-corasick-1.1.4/src/ahocorasick.rs|........A=Rkf0|/.cargo/
b|5b557f/tokio-1.48.0/src/runtime/context/current.rs|.......A=Rjg0|inconsistent
e|d8b9a47ec610a39/library/alloc/src/collections/btree/navigate.rs|.........e](1k
6cFzu3|   [... omitted |.ZY>][|rameZY>]le?QAp0bMvU4|rary/std/src/alloc.rs|.A@X:v
6|76/src/symbolize/gimli/stash.rs|.e](1k0|~/.cargo/registry/src/index.crat
c|es.io-1949cf8c6b5b557f/gimli-0.32.3/src/read/abbrev.rs|........A=Ri}0|START_GR
4|3/src/meta/limited.rs|.A@W805|hortest pattern length: |...ZYs=x4Sb=]0|ory usag
7|o-corasick-1.1.4/src/util/debug.rs|...A=Rjl0|tried to unwrap byte class f
8|/memchr-2.7.6/src/arch/all/twoway.rs|......0dDNN0|cargo/registry/src/index
1|level.rs02YiO4|orting due to panic at vrrTc0Y*X70sw?x2oUAH| << ZYt>:0|~/.cargo
6|r-0.3.22/src/registry/stack.rs|..A=Rkf0|/.cargo/registry/src/index.crate
c|s.io-1949cf8c6b5b557f/addr2line-0.25.1/src/function.rs|........A=Rkf0|/.rustup
a|lib/src/rust/library/core/src/num/wrapping.rs|.......A@Vz!|niond8k<j077g7|nnot
c|9cf8c6b5b557f/aho-corasick-1.1.4/src/util/primitives.rs|.......e](1k0|~/.rustu
e|tlib/src/rust/library/std/src/sys/thread_local/native/lazy.rs|...........A@VHN
5|ddr2line-0.25.1/src/line.rs|e](1k0|/rust/deps/rustc-demangle-0.1.26/src
|/libe](1k5|library/core/src/fmt/mod.rs|e](1k0|library/core/src/str/pattern
e](1k7|library/core/src/slice/memchr.rs|.....0bMvU0|rary/core/src/num/flt2de
4|c/strategy/dragon.rs|..01%rX3|nge start index |.Z-<[t0|ut of range for slic
c|557f/tracing-subscriber-0.3.22/src/fmt/time/datetime.rs|.......e](1k6LRDd|alid
4|ions/btree/map/entry.rse](1k3Rs(u1|ntifier(ZYt>:Z=iDJ0|s both a start and a
8|ex-automata-0.4.13/src/meta/error.rs|......06Cab0|x number of byte-based e
2|8.8/src/utf8.e](1k5|library/std/src/io/mod.rs|..A@W>l0|ust/deps/addr2line-0
3|.25.1/src/unit.rs|A@W>l0|ust/deps/rustc-demangle-0.1.26/src/legacy.rs
0bMvU4|rary/alloc/src/sync.rs|A=Rkf0|/.cargo/registry/src/index.crates.io
7|utomata-0.4.13/src/dfa/remapper.rs|...A=Rkf0|/.cargo/registry/src/index.c
|s.rs0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rege
7|x-automata-0.4.13/src/util/wire.rs|...A=Rkf0|/.cargo/registry/src/index.c
9|gex-automata-0.4.13/src/meta/strategy.rs|.......05Ow20|ap usage during NFA 
1|teddy.rs02/u/5|eating a new thread ID (|...Z^fy(0|would exceed the maximum
8|rust/library/std/src/thread/local.rs|......0kMy=0Y?Et0dDNN0|cargo/registry/s
5|a/thompson/range_trie.rs|...07QKq0|mpiling DFA with total patterns in a
7|tomata-0.4.13/src/meta/literal.rs|....A@ZUx0|.cargo/registry/src/index.cr
d|ates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/imp_std.rs|...........0bMvU|rary
b|/std/src/sys/pal/unix/stack_overflow/thread_info.rs|......e](1k0|~/.cargo/reg
6|3.22/src/registry/extensions.rs|.e](1k0|~/.cargo/registry/src/index.crat
b|es.io-1949cf8c6b5b557f/dotenv-0.15.0/src/parse.rs|........A@ZUx0|.cargo/regis
5|acktrace-0.3.76/src/lib.rs|.A=Rkf0|/.cargo/registry/src/index.crates.io
e|-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/metrics/worker.rs|............0bMvU
5|rary/std/src/io/stdio.rs|...01r*P1|nicked aBrFv2iTSUU3uP%40|ead panicked whi
e|ndex.crates.io-1949cf8c6b5b557f/gimli-0.32.3/src/read/index.rs|..........A=Rkf
8|mata-0.4.13/src/nfa/thompson/pikevm.rs|....A=Rkf0|/.cargo/registry/src/ind
3|filter/memmem.rs|.0dDNN0|cargo/registry/src/index.crates.io-1949cf8c6
c|b5b557f/regex-automata-0.4.13/src/meta/reverse_inner.rs|.......e](1k7?)(h|alid
c|-1949cf8c6b5b557f/tokio-1.48.0/src/util/linked_list.rs|........A=Rkf0|/.cargo/
a|o-1949cf8c6b5b557f/smallvec-1.15.1/src/lib.rs|.......A@ZUx0|.cargo/registry/
3|olize/gimli/elf.rsA=Rkf0|/.cargo/registry/src/index.crates.io-1949cf8
8|c6b5b557f/gimli-0.32.3/src/read/line.rs|...e](1k0|Acompiling DFA with star
1|_list.rs0bMvUb|rary/std/src/sys/thread_local/destructors/list.rs|........A@VOS
c|linux-gnu/lib/rustlib/src/rust/library/core/src/cell.rs|.......e](1k0|~/.cargo
4|4.13/src/hybrid/dfa.rs|A=Ri?2|START(ALL): |ZYs=x5OVca2|alid 'to' id:iV#ir|~/.c
4|0.8.8/src/unicode.rs|..055wn0|stc/c86564c412a5949088a53b665d8b9a47ec61
8|0a39/library/core/src/ops/function.rs|.....A@W>l0|ust/deps/miniz_oxide-0.8
6|.9/src/inflate/output_buffer.rs|.e](1k0|library/core/src/num/flt2dec/mod
b|-gnu/lib/rustlib/src/rust/library/std/src/io/mod.rs|......e](1k0|~/.rustup/to
7|/src/rust/library/std/src/env.rs|.....02/M?5|valid filter directive: |...ZYn*p
3|2/src/builders.rs|A@ZUx0|.cargo/registry/src/index.crates.io-1949cf8c
c|6b5b557f/regex-automata-0.4.13/src/nfa/thompson/nfa.rs|........A=Ri}0|attempte
y?%8/8|ibrary/std/src/sys/sync/rwlock/futex.rs|...e](1k0|library/std/src/thre
1|ad/mod.rA@Z2<6|brary/core/src/num/diy_float.rs|.e](1k00000%nSc0%nSc0%nSc0%nSc0
0|nt crates/jeb/src/bin/jeb.rs:54jebmessagecrates/jeb/src/bin/jeb.
b|cf8c6b5b557f/sharded-slab-0.1.7/src/tid.rs:163:21|........f:./p0|te: we were 
9|557f/sharded-slab-0.1.7/src/tid.rs:163:21|......f:./p0|te: we were already 
0|tracing-subscriber-0.3.22/src/registry/sharded.rs/!\ Tried to re
1x4K%00000o=qwIJxClMGtnr{I:!mHI9RW-5c+Ah5DLK:1y$5.6V9I51x4%NjJy3C.rsBF|l$ H%ikT!
oO@GbCZ/X[n=IjcnvY0a000000dWQ#l)KEA|\$8HiMI.+cLuyVpk}x$00000oO}W#.rs2)Mb2}y.sQqV
