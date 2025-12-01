#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname "$0")/setup"

cargo_flags=(--release)
jeb self split-64 encode-jeb85 find-\| find-.rs first-256 join-lines stdout

6|.3.22/src/filter/env/field.rs|...A@VLY1|gnoring v88>LiV#is3ikJ+0|argo/registr
a|b557f/aho-corasick-1.1.4/src/packed/pattern.rs|......A=Rkf0|cargo/registry/s
6|.48.0/src/runtime/task/state.rs|.e](1k0|/rustc/c86564c412a5949088a53b665
a|d8b9a47ec610a39/library/alloc/src/vec/mod.rs|........0bMvU0|rary/std/src/../
8|../backtrace/src/symbolize/gimli/elf.rs|...e](1k6(vJ03|ory allocation of w*1(0
5|-syntax-0.8.8/src/error.rs|.A=Rj<4|rates/jeb/src/z85.rs|..00aZ/ZYng+0|brary/st
5|d/src/sys/pal/unix/os.rs|...055wn0|st/deps/gimli-0.32.3/src/read/abbrev
9|tomata-0.4.13/src/util/prefilter/memchr.rs|.....A=Ri}0|attempted to compile
|i.rs0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557f/aho-co
7|rasick-1.1.4/src/nfa/contiguous.rs|...A=Rkf0|cargo/registry/src/index.cra
7|-corasick-1.1.4/src/automaton.rs|.....055wn0|st/deps/rustc-demangle-0.1.2
1|6/src/v0e](1k5|library/alloc/src/fmt.rs|...01-NW2|sertion `leftBrFvn0| right` 
8|lloc/src/collections/btree/map/entry.rs|...e](1kZYuHi01:9T2|ttern length:iV#is
b|7f/regex-automata-0.4.13/src/nfa/thompson/map.rs|.........0263O0|ystack of le
5|a-0.4.13/src/hybrid/dfa.rs|.A=Ri$5|longest pattern length: |...ZYs=x0|library/
8|std/src/io/buffered/linewritershim.rs|.....A@V}.4|at` split index (is |..Z.XBx
b|-1949cf8c6b5b557f/rustc-demangle-0.1.26/src/lib.rs|.......A=Rkf0|rustup/toolc
8|c/rust/library/core/src/ops/function.rs|...e](1k0|&number of DFA states ex
b|b557f/regex-automata-0.4.13/src/nfa/thompson/nfa.rs|......e](1k0|~cargo/regis
5|rc/util/prefilter/memmem.rs|e](1k4pT3?1|ary-uniozyAp4efI.edf72}0|nvalid 'from
1|habet.rs01-^Y2|t codepoint Urv+<i0| which occurs before last codepoint 
5|.48.0/src/sync/notify.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
d|9cf8c6b5b557f/tokio-1.48.0/src/runtime/metrics/worker.rs|...........0dFA^|rgo/
3|untime/handle.rs|.04zI*8|iled to allocate an alternative stack: |...yDrPN01imY
5|li-0.32.3/src/read/line.rs|.A=Ri]3|invalid pattern IDnKKYp0dFA^0|rgo/registry
5|nfa/thompson/compiler.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
c|9cf8c6b5b557f/regex-automata-0.4.13/src/util/utf8.rs|..........00^7Q|ror:aIW$i
e|49cf8c6b5b557f/tokio-1.48.0/src/runtime/blocking/shutdown.rs|............0bMvU
5|rary/std/src/ffi/os_str.rs|.A=Rjl0|rust/deps/addr2line-0.25.1/src/funct
|ion.A=Rj%8|ibrary/std/src/sync/reentrant_lock.rs|.....A@Vh?0|rustup/toolchain
6|st/library/std/src/sync/once.rs|.e](1k0|~cargo/registry/src/index.crates
b|.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs|.........0dFA^0|rgo/registry
2|util/look.rs|03VG67|arse set capacity cannot exceed |.....ZYn*[0|argo/registr
3|/meta/limited.rs|.0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5
c|b557f/regex-automata-0.4.13/src/util/prefilter/teddy.rs|.......e](1k3VHb$|filt
7|io-1.48.0/src/util/sharded_list.rs|...A=Rjl0|rustc/c86564c412a5949088a53b
7|/src/collections/btree/map/entry.rs|..e](1k0|/rustc/c86564c412a5949088a53
a|b665d8b9a47ec610a39/library/core/src/time.rs|........0bMvU0|rary/std/src/../
9|../backtrace/src/symbolize/gimli/stash.rs|......A@WX30|xtension cannot cont
4|ain path separators: |.aIW$[5|ibrary/core/src/fmt/num.rs|.A=Rj<0|rates/jeb/sr
2|c/bin/jeb.rs|0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557
7|f/color-spantrace-0.3.0/src/lib.rs|...A=Ri%4|tried to drop a ref to Bz&qq8z01u
6|tomata-0.4.13/src/dfa/search.rs|.e](1k0|~cargo/registry/src/index.crates
d|.io-1949cf8c6b5b557f/aho-corasick-1.1.4/src/ahocorasick.rs|.........A=Ri:|Span
e](1k8|library/core/src/num/dec2flt/parse.rs|.....A@WK%0|opy_from_slice: sour
a|ubscriber-0.3.22/src/filter/env/directive.rs|........03bGT0|celerator alread
b|5b557f/aho-corasick-1.1.4/src/util/primitives.rs|.........04A2g0|ied to unwra
c|-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/time/mod.rs|........A=Rjl0|rust/dep
6|s/gimli-0.32.3/src/read/line.rs|.e](1k0|library/std/src/../../backtrace/
4|src/symbolize/mod.rs|..01imY1|te indexaIX<q5|is not an OsStr boundary|...055wn
8|st/deps/hashbrown-0.16.1/src/raw/mod.rs|...e](1k0|library/alloc/src/raw_ve
1|c/mod.rs01%r-3|moval index (is |.Z.Ovw4|should be < len (is |..ZYt>:0|library/
5|core/src/fmt/builders.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
c|9cf8c6b5b557f/regex-automata-0.4.13/src/util/pool.rs|..........03kY(0|ror: unr
|d.rs0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557f/backtr
6|ace-0.3.76/src/symbolize/mod.rs|.e](1k0|*anchored searches for a specifi
3|determinize/mod.rsA=Rkf0|cargo/registry/src/index.crates.io-1949cf8c6
a|b5b557f/regex-automata-0.4.13/src/dfa/accel.rs|......A=RjC0|internal error: 
7|0/src/runtime/context/runtime.rs|.....0dFA^0|rgo/registry/src/index.crate
d|s.io-1949cf8c6b5b557f/tokio-1.48.0/src/util/linked_list.rs|.........A=Rj%|ibra
b|ry/std/src/../../backtrace/src/symbolize/gimli.rs|........A@YrD0|tracing-subs
c|557f/tracing-subscriber-0.3.22/src/filter/directive.rs|........A=Rkf0|cargo/re
3|c/inflate/core.rs|A@ZV00|argo/registry/src/index.crates.io-1949cf8c6b
b|5b557f/regex-automata-0.4.13/src/hybrid/regex.rs|.........04&qk0|ied to unwra
c|949cf8c6b5b557f/tracing-core-0.1.35/src/dispatcher.rs|.........A@ZVf0|ustup/to
8|/src/rust/library/core/src/cell/once.rs|...e](1k0|~rustup/toolchains/night
9|ary/alloc/src/collections/btree/navigate.rs|....e](1kZYu451rY&>aIW#a0|cargo/re
6|3/src/nfa/thompson/pikevm.rs|....0dFA^0|rgo/registry/src/index.crates.io
d|-1949cf8c6b5b557f/regex-automata-0.4.13/src/util/wire.rs|...........0dFA^|rgo/
8|557f/tokio-1.48.0/src/util/wake_list.rs|...e](1k0|~cargo/registry/src/inde
1|/level.rA@Z2<7|brary/std/src/sys/thread/unix.rs|.....0bMvU0|rary/std/src/sys
5|/pal/unix/stack_overflow.rs|e](1k7isqa3|ce index starts atvrrTc4iPb50|t ends a
BrFv06|library/core/src/str/lossy.rs|...A@ZV00|argo/registry/src/index.crat
b|x-gnu/lib/rustlib/src/rust/library/std/src/env.rs|........A@WX70|nternal erro
d|io-1949cf8c6b5b557f/regex-automata-0.4.13/src/dfa/dense.rs|.........A=Rkf|carg
4|0/src/inline_lazy.rs|..0dF%h0|stup/toolchains/nightly-2025-11-28-x86_6
4|ctions/btree/node.rs|..0dFA^0|rgo/registry/src/index.crates.io-1949cf8
9|c6b5b557f/tracing-core-0.1.35/src/field.rs|.....A=Rkf0|cargo/registry/src/i
1|scape.rs0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557f/re
9|gex-automata-0.4.13/src/util/primitives.rs|.....A=Rkf0|cargo/registry/src/i
e|ndex.crates.io-1949cf8c6b5b557f/aho-corasick-1.1.4/src/dfa.rs|...........A@Z2<
7|brary/std/src/sys/pal/unix/mod.rs|....A@Z2<5|brary/core/src/panicking.rs|e](1k
8|57f/addr2line-0.25.1/src/function.rs|......0dFA^0|rgo/registry/src/index.c
8|r-2.7.6/src/arch/generic/packedpair.rs|....A=Ri?2|capture(pid=|ZZ8Fl|grouA6p#w
b|557f/regex-automata-0.4.13/src/dfa/determinize.rs|........A@Wa)0|nvalid accel
8|b5b557f/regex-syntax-0.8.8/src/utf8.rs|....A=Ri]3|failed printing toBz&qq0Y?Et
0bMvU6|rary/std/src/sync/lazy_lock.rs|..A=Rj%0|ibrary/std/src/sys/random/li
|nux.A=Rj%4|ibrary/std/src/time.rs|A=Rj60|index out of bounds: the len is 
e|8c6b5b557f/tracing-subscriber-0.3.22/src/registry/extensions.rs|.........e](1k
6|scriber-0.3.22/src/fmt/mod.rs|...A@ZV00|argo/registry/src/index.crates.i
5|.4.13/src/util/alphabet.rs|.A=Ri^1|on line ZZhbi|coluzeHGi4>eSy0|hrough line 
ZZhbi|coluzeHGi0vX1U4|rates/jeb/src/jeb85.rs|A=Rkf0|cargo/registry/src/index
4|core/src/cell/once.rs|.A@W>l0|ustc/c86564c412a5949088a53b665d8b9a47ec6
8|10a39/library/core/src/num/wrapping.rs|....A=Rj%4|ibrary/std/src/rt.rs|..0bMvU
6|rary/std/src/thread/current.rs|..A=Rj%0|ibrary/core/src/unicode/printabl
|e.rs0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557f/tracin
a|g-subscriber-0.3.22/src/fmt/format/pretty.rs|........04]{m0|rates/jeb/src/er
4|llvec-1.15.1/src/lib.rse](1k0|~cargo/registry/src/index.crates.io-1949
d|cf8c6b5b557f/backtrace-0.3.76/src/symbolize/gimli/elf.rs|...........0dF%h|stup
a|lib/src/rust/library/core/src/num/wrapping.rs|.......A@ZV00|argo/registry/sr
9|tomata-0.4.13/src/nfa/thompson/builder.rs|......A@ZV00|argo/registry/src/in
3|terminize/state.rsA=Rkf0|cargo/registry/src/index.crates.io-1949cf8c6
b|b5b557f/regex-automata-0.4.13/src/meta/strategy.rs|.......A=Rkf0|cargo/regist
3|c/dfa/onepass.rs|.02Po(4|pected char at offset |BrFv00|~cargo/registry/src/
5|48.0/src/runtime/park.rs|...0bMvU5|rary/std/src/panicking.rs|..A@ZV00|argo/reg
6|c-demangle-0.1.26/src/legacy.rs|.e](1k0|9internal error: entered unreach
5|fa/thompson/range_trie.rs|..A@ZV00|argo/registry/src/index.crates.io-19
d|49cf8c6b5b557f/regex-automata-0.4.13/src/util/captures.rs|..........A@ZV0|argo
3|memmem/searcher.rsA=Rkf0|cargo/registry/src/index.crates.io-1949cf8c6
e|b5b557f/regex-automata-0.4.13/src/nfa/thompson/literal_trie.rs|..........A=RiZ
B-O#i3@y>-1|ition(o:aIXdi| l: ZY=niv(+:F0vX1+4|ibrary/std/src/path.rs|A=Rjl|rust
8|/deps/gimli-0.32.3/src/read/index.rs|......055wn0|st/deps/miniz_oxide-0.8.
4|9/src/inflate/core.rs|.A@Z2<6|brary/core/src/num/bignum.rs|....0bMvU0|rary/cor
7|e/src/num/dec2flt/decimal_seq.rs|.....0dFA^0|rgo/registry/src/index.crate
0|index.crates.io-1949cf8c6b5b557f/gimli-0.32.3/src/read/abbrev.rs
6|k-1.1.4/src/packed/rabinkarp.rs|.e](1k0|*tried to unwrap group from HirF
5|ibrary/std/src/sync/once.rs|e](1k0|/rustc/c86564c412a5949088a53b665d8b9
c|a47ec610a39/library/alloc/src/collections/btree/node.rs|.......e](1k0|/rustc/c
1|, abortizF6pR8|library/std/src/sys/pal/unix/time.rs|......02ouZ0|mory allocat
9|f8c6b5b557f/sharded-slab-0.1.7/src/tid.rs|......A@-#RaohHGB:H[8Ee[.a0|argo/reg
5|symbolize/gimli/stash.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
9|9cf8c6b5b557f/addr2line-0.25.1/src/line.rs|.....A=Rjd0|determinization exce
b|c6b5b557f/aho-corasick-1.1.4/src/util/remapper.rs|........A@ZV00|argo/registr
d|b557f/tracing-subscriber-0.3.22/src/fmt/time/datetime.rs|...........0dFA^|rgo/
5|rc/symbolize/gimli/lru.rs|..A@ZV00|argo/registry/src/index.crates.io-19
9|49cf8c6b5b557f/addr2line-0.25.1/src/unit.rs|....e](1k0|~cargo/registry/src/
4|prefilter/byteset.rs|..0dFA^0|rgo/registry/src/index.crates.io-1949cf8
c|c6b5b557f/regex-automata-0.4.13/src/hybrid/search.rs|..........04IY20|consiste
A@V<@3|     [... omitted wfwU(| frazdQG!| ...t)ZNF4|ibrary/std/src/alloc.rse](1k
b|rustlib/src/rust/library/core/src/str/pattern.rs|.........0dF%h0|stup/toolcha
c|rust/library/core/src/slice/sort/stable/quicksort.rs|..........0dFA^0|rgo/regi
5|.22/src/registry/stack.rs|..A@V}N4|TART_GROUP(pattern: |..ZYC}(04A2d0|o many p
d|cf8c6b5b557f/aho-corasick-1.1.4/src/nfa/noncontiguous.rs|...........02Y&>|orte
4|c/util/sparse_set.rs|..00WN-|ion(ZYt>:0|~cargo/registry/src/index.crates
d|.io-1949cf8c6b5b557f/regex-automata-0.4.13/src/dfa/regex.rs|........e](1k|Bcan
d|49cf8c6b5b557f/regex-automata-0.4.13/src/meta/literal.rs|...........0dFA^|rgo/
3|/src/util/debug.rsA=Rkf0|rustup/toolchains/nightly-2025-11-28-x86_64-
4|d_local/native/lazy.rs|A=Ri!3uP%4|ead c+]*iasL2A0|) has overflowed its sta
v}#zK4|library/std/src/env.rs|A=Rjl0|failed to set up alternative stack g
1|uard pagwJy%H055wn8|st/deps/addr2line-0.25.1/src/line.rs|......055wn0|st/deps/
7|rustc-demangle-0.1.26/src/lib.rs|.....0bMvU5|rary/core/src/fmt/mod.rs|...0bMvU
6|rary/core/src/str/pattern.rs|....0bMvU6|rary/core/src/slice/memchr.rs|...A@Z2<
a|brary/core/src/num/flt2dec/strategy/dragon.rs|.......A@V?[0|ange start index
4|ry/alloc/src/string.rs|A=Ri{4|invalid field name `|..ZYv*y0|~cargo/registry/
|n.rs01hwe1|entifierd8k<j0kHMp0|is both a start and a match state, which
2|xt/current.rsA@Z2<4|brary/std/src/io/mod.rse](1k0|/rust/deps/addr2line-0.2
2|5.1/src/unit.e](1ka|/rust/deps/rustc-demangle-0.1.26/src/legacy.rs|......A=Rj%
5|ibrary/alloc/src/sync.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
c|9cf8c6b5b557f/regex-automata-0.4.13/src/util/search.rs|........A=Rkf0|cargo/re
4|3/src/meta/wrappers.rs|A=Rjq0|heap usage during NFA compilation exceed
7|7f/tokio-1.48.0/src/util/rand/rt.rs|..e](1k8E)/04|ating a new thread ID (l)9[[
5|otenv-0.15.0/src/parse.rs|..A@ZV00|argo/registry/src/index.crates.io-19
a|49cf8c6b5b557f/color-eyre-0.6.5/src/config.rs|.......A@:1>ZYDLtZYn*[0|argo/reg
a|57f/regex-automata-0.4.13/src/dfa/remapper.rs|.......A@XZu0|ompiling DFA wit
b|8c6b5b557f/regex-automata-0.4.13/src/util/empty.rs|.......A=Rkf0|rustup/toolc
6|c/rust/library/core/src/cell.rs|.e](1k0|~cargo/registry/src/index.crates
d|.io-1949cf8c6b5b557f/regex-syntax-0.8.8/src/hir/literal.rs|.........A=Rkf|carg
b|tracing-subscriber-0.3.22/src/registry/sharded.rs|........A@ZVf0|ustup/toolch
8|/rust/library/std/src/thread/local.rs|.....A@ZV00|argo/registry/src/index.
9|tomata-0.4.13/src/nfa/thompson/backtrack.rs|....e](1k0|~cargo/registry/src/
1|error.rs0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557f/ah
a|o-corasick-1.1.4/src/packed/teddy/generic.rs|........0bMvU0|rary/std/src/io/
1|stdio.rs01r*P1|nicked aBrFv2iTSUU3uP%40|ead panicked while processing pa
2|nic. abortingxd:I>a|library/core/src/slice/sort/shared/smallsort.rs|.....e](1k
9|9cf8c6b5b557f/tracing-log-0.2.0/src/lib.rs|.....A=Rkf0|cargo/registry/src/i
9|557f/sharded-slab-0.1.7/src/page/slot.rs|.......0dF%h0|stup/toolchains/nigh
4|rary/std/src/io/mod.rs|A=Rkf0|rustup/toolchains/nightly-2025-11-28-x86
5|-slab-0.1.7/src/page/mod.rs|e](1k2oUci| at ZYuHi00Lhsj2e[k0xX6$jhp?1|38;2jhp>U
7|f/gimli-0.32.3/src/read/index.rs|.....0dF%h0|stup/toolchains/nightly-2025
a|b5b557f/aho-corasick-1.1.4/src/util/search.rs|.......A@ZV00|argo/registry/sr
7|o-1.48.0/src/runtime/time/entry.rs|...A=Rkf0|rustup/toolchains/nightly-20
4|td/src/thread/mod.rs|..0bMvU0|rary/std/src/sys/thread_local/destructor
4|backtrace/libunwind.rs|A=Ri<2|state length:iV#is3ihn(0|oo many capture grou
5|/src/meta/reverse_inner.rs|.A=Rkf0|rustup/toolchains/nightly-2025-11-28
|on.rA@W>lc|ust/deps/miniz_oxide-0.8.9/src/inflate/output_buffer.rs|.......e](1k
7|library/core/src/num/flt2dec/mod.rs|..e](1k8FK8j4|alid filter directive: wJy%H
6|mata-0.4.13/src/meta/stopat.rs|..A=Ri}4|attempted to compile |.aIYxF0|NFA stat
e|s.io-1949cf8c6b5b557f/regex-syntax-0.8.8/src/hir/interval.rs|............054>]
|v/nuy?%8/8|ibrary/std/src/sys/sync/rwlock/futex.rs|...e](1k0|library/std/src/
2|thread/mod.rsA@Z2<6|brary/core/src/num/diy_float.rs|.e](1k%nSc0%nSc0%nSc0%nSc0
0|nt crates/jeb/src/bin/jeb.rs:54jebmessagecrates/jeb/src/bin/jeb.
a|8c6b5b557f/sharded-slab-0.1.7/src/tid.rs:163:21|.....iX&z%0|note: we were al
8|7f/sharded-slab-0.1.7/src/tid.rs:163:21|...iX&z%0|note: we were already un
0|ing-subscriber-0.3.22/src/registry/sharded.rs/!\ Tried to regist
Fb/MHn@TKY00000oO}W#.rs5[Mb2@3@qV/%%nSaGnia*6M+-%znq?NnndQzW2MK&}I/TblH0&IG|\$@L
Gtnr{I:!mHI9RW-5c+Ah5DLK:1y$5.6V9I51x4%NjJy3C.rsBF|l$ H%ikT!nqRddawvXpmkrjVg22lQ
000000dWQ#l)KEA|\$8HiMI.+cLuyVpk}x$00000oO}W#.rs2)Mb2}y.sQqV=KVwU2/d48NhC#8mksg6
