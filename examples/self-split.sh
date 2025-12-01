#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
source "$(dirname "$0")/setup"

cargo_flags=(--release)
jeb self split-64 encode-jeb85 find-\| find-.rs first-256 join-lines stdout

e|f8c6b5b557f/tracing-subscriber-0.3.22/src/filter/env/field.rs|...........A@VLY
7|5b557f/regex-1.12.2/src/builders.rs|..e](1k2Y>9d|ide:aIX0+0dFA^0|rgo/registry
7|57f/once_cell-1.21.3/src/imp_std.rs|..e](1k0|~cargo/registry/src/index.cr
e|ates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/task/state.rs|.........e](1k
2|c/vec/mod.rs|0bMvU0|rary/std/src/../../backtrace/src/symbolize/gimli
8|src/rust/library/alloc/src/vec/mod.rs|.....A@ZV00|argo/registry/src/index.
d|crates.io-1949cf8c6b5b557f/regex-syntax-0.8.8/src/error.rs|.........A=Rj<|rate
3|s/jeb/src/z85.rs|.00aZ/ZYng+7|brary/std/src/sys/pal/unix/os.rs|.....055wn|st/d
7|eps/gimli-0.32.3/src/read/abbrev.rs|..e](1k5mGT32|ertion `left aIXKh0|right` f
8|aho-corasick-1.1.4/src/packed/api.rs|......0dFA^0|rgo/registry/src/index.c
7|48.0/src/runtime/time/wheel/mod.rs|...A=Ri>2|prelude errorAYLtU06%N$0|read cou
0|x.crates.io-1949cf8c6b5b557f/aho-corasick-1.1.4/src/automaton.rs
055wn8|st/deps/rustc-demangle-0.1.26/src/v0.rs|...e](1k0|library/alloc/src/fm
|t.rs01-NW2|sertion `leftBrFvn2| right` failewN(b<| lefBugLW2)}!}|ightiV#ir0u/eU
3|/thompson/map.rs|.0263O3|ystack of length |aIXyd1|is too lz/fg/0|#expected va
7|tomata-0.4.13/src/dfa/minimize.rs|....A@WdE5|FA exceeded size limit of |.w*1(9
d|o-1949cf8c6b5b557f/regex-automata-0.4.13/src/hybrid/dfa.rs|.........A=Ri$|long
3|-0.1.26/src/lib.rsA=Rkf0|rustup/toolchains/nightly-2025-11-28-x86_64-
d|6b5b557f/regex-automata-0.4.13/src/util/prefilter/memmem.rs|........e](1k4pT3?
9|/aho-corasick-1.1.4/src/util/alphabet.rs|.......01-^Y2|t codepoint Urv+<i| whi
d|ates.io-1949cf8c6b5b557f/tokio-1.48.0/src/sync/notify.rs|...........0dFA^|rgo/
5|untime/metrics/worker.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
b|9cf8c6b5b557f/tokio-1.48.0/src/runtime/handle.rs|.........04zI*0|iled to allo
d|x.crates.io-1949cf8c6b5b557f/gimli-0.32.3/src/read/line.rs|.........A=Ri]|inva
d|b557f/regex-automata-0.4.13/src/nfa/thompson/compiler.rs|...........0dFA^|rgo/
4|.13/src/util/utf8.rs|..00^7Q|ror:aIW$i0|exceeded the maximum number of c
6|runtime/blocking/shutdown.rs|....0bMvU5|rary/std/src/ffi/os_str.rs|.A=Rjl|rust
8|/deps/addr2line-0.25.1/src/function.rs|....A=Rj%0|ibrary/std/src/sync/reen
2|trant_lock.rsA@Vh?0|rustup/toolchains/nightly-2025-11-28-x86_64-unkn
e|own-linux-gnu/lib/rustlib/src/rust/library/std/src/sync/once.rs|.........e](1k
3|.21.3/src/lib.rs|.0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5
a|b557f/regex-automata-0.4.13/src/util/look.rs|........03VG60|arse set capacit
b|5b557f/regex-automata-0.4.13/src/meta/limited.rs|.........0dFA^0|rgo/registry
4|util/prefilter/teddy.rse](1k3VHb$1|filter: ZYs=x0|~cargo/registry/src/inde
7|src/slice/sort/stable/quicksort.rs|...A=Rjl0|rustc/c86564c412a5949088a53b
2|/src/time.rs|0bMvU0|rary/std/src/../../backtrace/src/symbolize/gimli
3|ore/src/fmt/num.rsA=Rj<5|rates/jeb/src/bin/jeb.rs|...0dFA^0|rgo/registry/src
e|tes.io-1949cf8c6b5b557f/regex-automata-0.4.13/src/dfa/search.rs|.........e](1k
5|k-1.1.4/src/ahocorasick.rs|.A=Ri:|Spand8k)naIX1b0bMvU0|rary/std/src/../../b
7|acktrace/src/symbolize/gimli/lru.rs|..e](1k0|library/core/src/num/dec2flt
2|directive.rs|03bGT5|celerator already contains |zGu@o0kFQ!ZYn*[0|argo/registr
3|il/primitives.rs|.04A2g8|ied to unwrap expr from HirFrame, got: |...BugLW0dFA^
4|rc/runtime/time/mod.rs|A=Rjl8|rust/deps/gimli-0.32.3/src/read/line.rs|...e](1k
c|library/std/src/../../backtrace/src/symbolize/mod.rs|..........01imY0|te index
|/mode](1k7|library/alloc/src/raw_vec/mod.rs|.....01%r-3|moval index (is |.Z.Ovw
4|should be < len (is |..ZYt>:7|library/core/src/fmt/builders.rs|.....0dFA^|rgo/
4|.13/src/util/pool.rs|..03kY(6|ror: unrecognized argument: |....ZYs=xZYjxxZYjSE
8|/thread_local-1.1.9/src/thread_id.rs|......0dFA^0|rgo/registry/src/index.c
e|rates.io-1949cf8c6b5b557f/backtrace-0.3.76/src/symbolize/mod.rs|.........e](1k
b|/regex-automata-0.4.13/src/util/determinize/mod.rs|.......A=Rkf0|cargo/regist
9|5b557f/regex-syntax-0.8.8/src/ast/parse.rs|.....A=Rjd0|haystack too small, 
0|.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/runtime.rs
5|.0/src/util/linked_list.rs|.A=Rj%0|ibrary/std/src/../../backtrace/src/s
3|ymbolize/gimli.rs|A@YrD0|tracing-subscriber] Unable to write an event
4|rc/filter/directive.rs|A=Rkf0|cargo/registry/src/index.crates.io-1949c
b|f8c6b5b557f/miniz_oxide-0.8.9/src/inflate/core.rs|........A@ZV00|argo/registr
3|/hybrid/regex.rs|.04&qk9|ied to unwrap alt pipe from HirFrame, got: |....BugLW
8|ibrary/std/src/sys/io/io_slice/iovec.rs|...e](1kZZznk1|os erroraIX1b0bMvU|rary
8|/core/src/num/flt2dec/strategy/grisu.rs|...e](1k0|E[tracing-subscriber] Un
4|.35/src/dispatcher.rs|.A@ZVf0|ustup/toolchains/nightly-2025-11-28-x86_
e|f8c6b5b557f/regex-automata-0.4.13/src/nfa/thompson/pikevm.rs|............0dFA^
5|-0.4.13/src/util/wire.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
a|9cf8c6b5b557f/regex-syntax-0.8.8/src/unicode.rs|.....e](1k0|~cargo/registry/
9|io-1.48.0/src/runtime/time/wheel/level.rs|......A@Z2<0|brary/std/src/sys/th
2|read/unix.rs|0bMvU9|rary/std/src/sys/pal/unix/stack_overflow.rs|....e](1k7isqa
8|ubscriber-0.3.22/src/filter/env/mod.rs|....A=Rkf0|rustup/toolchains/nightl
3|ry/std/src/env.rs|A@WX79|nternal error: entered unreachable code: |......aIW#a
5|ta-0.4.13/src/dfa/dense.rs|.A=Rkf0|cargo/registry/src/index.crates.io-1
c|949cf8c6b5b557f/lazy_static-1.5.0/src/inline_lazy.rs|..........0dF%h0|stup/too
c|src/rust/library/alloc/src/collections/btree/node.rs|..........0dFA^0|rgo/regi
9|regex-automata-0.4.13/src/util/escape.rs|.......0dFA^0|rgo/registry/src/ind
6|aho-corasick-1.1.4/src/dfa.rs|...A@Z2<0|brary/std/src/sys/pal/unix/mod.r
A@Z2<5|brary/core/src/panicking.rs|e](1k0|~cargo/registry/src/index.crates
a|.io-1949cf8c6b5b557f/matchers-0.2.0/src/lib.rs|......A=Rkf0|cargo/registry/s
|n.rs0dFA^0|rgo/registry/src/index.crates.io-1949cf8c6b5b557f/memchr
8|-2.7.6/src/arch/all/packedpair/mod.rs|.....A@ZV00|argo/registry/src/index.
3|fa/determinize.rs|A@Wa)5|nvalid accelerator index |..aIW%UfNgwk0|:exceed the 
A=Rj%7|ibrary/std/src/sys/random/linux.rs|...A=Rj%4|ibrary/std/src/time.rs|A=Rj6
6|3.22/src/registry/extensions.rs|.e](1k0|~cargo/registry/src/index.crates
e|.io-1949cf8c6b5b557f/tracing-subscriber-0.3.22/src/fmt/mod.rs|...........A@ZV0
7|riber-0.3.22/src/fmt/fmt_layer.rs|....A@WE$0|ailed to set environment var
d|949cf8c6b5b557f/regex-automata-0.4.13/src/util/alphabet.rs|.........A=Ri^|on l
8|o-1.48.0/src/runtime/scheduler/defer.rs|...e](1k0|/rustc/c86564c412a594908
c|8a53b665d8b9a47ec610a39/library/core/src/cell/once.rs|.........A@W>l0|ustc/c86
|ing.A=Rj%4|ibrary/std/src/rt.rs|..0bMvU6|rary/std/src/thread/current.rs|..A=Rj%
8|ibrary/core/src/unicode/printable.rs|......0dFA^0|rgo/registry/src/index.c
2|at/pretty.rs|04]{m4|rates/jeb/src/errors.rse](1k0|~cargo/registry/src/inde
c|x.crates.io-1949cf8c6b5b557f/smallvec-1.15.1/src/lib.rs|.......e](1k0|~cargo/r
5|c/symbolize/gimli/elf.rs|...0dF%h0|stup/toolchains/nightly-2025-11-28-x
2|m/wrapping.rsA@ZV00|argo/registry/src/index.crates.io-1949cf8c6b5b55
7|7f/rustc-demangle-0.1.26/src/v0.rs|...A=Rkf0|cargo/registry/src/index.cra
b|egex-automata-0.4.13/src/util/determinize/state.rs|.......A=Rkf0|cargo/regist
3|c/meta/strategy.rsA=Rkf0|cargo/registry/src/index.crates.io-1949cf8c6
b|b5b557f/regex-automata-0.4.13/src/dfa/onepass.rs|.........02Po(0|pected char 
7|/regex-syntax-0.8.8/src/hir/mod.rs|...A=Rkf0|cargo/registry/src/index.cra
d|tes.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/park.rs|...........0bMvU|rary
4|/std/src/panicking.rs|.A@ZV00|argo/registry/src/index.crates.io-1949cf
8|8c6b5b557f/backtrace-0.3.76/src/lib.rs|....A=Rkf0|cargo/registry/src/index
e|.crates.io-1949cf8c6b5b557f/rustc-demangle-0.1.26/src/legacy.rs|.........e](1k
d|557f/regex-automata-0.4.13/src/nfa/thompson/range_trie.rs|..........A@ZV0|argo
5|4.13/src/util/captures.rs|..A@ZV00|argo/registry/src/index.crates.io-19
b|49cf8c6b5b557f/memchr-2.7.6/src/memmem/searcher.rs|.......A=Rkf0|cargo/regist
6|c/nfa/thompson/literal_trie.rs|..A=RiZasL2adf75(3|apture group indexwQ2uk|& is
0vX1+4|ibrary/std/src/path.rs|A=Rjl0|rust/deps/gimli-0.32.3/src/read/inde
|x.rs055wna|st/deps/miniz_oxide-0.8.9/src/inflate/core.rs|.......A@Z2<0|brary/co
4|re/src/num/bignum.rs|..0bMvU0|rary/core/src/num/dec2flt/decimal_seq.rs
a|.0/src/runtime/scheduler/current_thread/mod.rs|......A=Rk@0XtFfZZ8Ml|ourcwPLS*
7|/gimli-0.32.3/src/read/abbrev.rs|.....03(Y80|supported regex feature for 
e|.io-1949cf8c6b5b557f/aho-corasick-1.1.4/src/packed/rabinkarp.rs|.........e](1k
6|petition from HirFrame, got: |...aIW$[5|ibrary/std/src/sync/once.rs|e](1k|/rus
4|llections/btree/node.rse](1k0|/rustc/c86564c412a5949088a53b665d8b9a47e
8|c610a39/library/core/src/str/pattern.rs|...e](1k0|5fatal runtime error: fa
3|pal/unix/time.rs|.02ouZ3|mory allocation ofz!pcc4iPb91|tes failwN>^&0|cargo/re
d|8c6b5b557f/backtrace-0.3.76/src/symbolize/gimli/stash.rs|...........0dFA^|rgo/
3|/util/remapper.rs|A@ZV00|argo/registry/src/index.crates.io-1949cf8c6b
8|5b557f/sharded-slab-0.1.7/src/shard.rs|....A=Ri-|    ZYtN[0dFA^0|rgo/registry
5|src/fmt/time/datetime.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
d|9cf8c6b5b557f/backtrace-0.3.76/src/symbolize/gimli/lru.rs|..........A@ZV0|argo
c|/regex-automata-0.4.13/src/util/prefilter/byteset.rs|..........0dFA^0|rgo/regi
4|src/hybrid/search.rs|..04IY20|consistent park_timeout state; actual = 
7|src/collections/btree/navigate.rs|....A@V<@3|     [... omitted wfwU(| frazdQG!
| ...t)ZNF4|ibrary/std/src/alloc.rse](1k0|Hcannot access a Thread Local St
8|/backtrace/src/backtrace/libunwind.rs|.....A@V+(2|ange end indewQ2uk0|" out of
3|c/str/pattern.rs|.0dF%h0|stup/toolchains/nightly-2025-11-28-x86_64-un
4|/stable/quicksort.rs|..0dFA^0|rgo/registry/src/index.crates.io-1949cf8
d|c6b5b557f/tracing-subscriber-0.3.22/src/registry/stack.rs|..........A@V}N|TART
5|src/nfa/noncontiguous.rs|...02Y&>4|ortest pattern length: xGvoK0sw/p0|memory u
8|egex-syntax-0.8.8/src/hir/translate.rs|....A=Rjl0|tried to unwrap byte cla
7|7f/tokio-1.48.0/src/sync/oneshot.rs|..e](1k8du#)4|rting due to panic at |BrFv2
c|b5b557f/regex-automata-0.4.13/src/util/sparse_set.rs|..........00WN-|ion(ZYt>:
5|ata-0.4.13/src/dfa/regex.rs|e](1k0|Bcannot create iterator for StateID 
5|4.13/src/meta/literal.rs|...0dFA^0|rgo/registry/src/index.crates.io-194
b|9cf8c6b5b557f/aho-corasick-1.1.4/src/util/debug.rs|.......A=Rkf0|rustup/toolc
c|c/rust/library/std/src/sys/thread_local/native/lazy.rs|........A=Ri!3uP%4|ead 
c+]*iasL2A5|) has overflowed its stack|.v}#zK4|library/std/src/env.rs|A=Rjl|fail
4|e-0.25.1/src/line.rs|..055wn0|st/deps/rustc-demangle-0.1.26/src/lib.rs
0bMvU5|rary/core/src/fmt/mod.rs|...0bMvU6|rary/core/src/str/pattern.rs|....0bMvU
6|rary/core/src/slice/memchr.rs|...A@Z2<0|brary/core/src/num/flt2dec/strat
2|egy/dragon.rsA@V?[3|ange start index |aIYfz0|out of range for slice of le
c|x-gnu/lib/rustlib/src/rust/library/alloc/src/string.rs|........A=Ri{0|invalid 
8|557f/tracing-core-0.1.35/src/span.rs|......01hwe1|entifierd8k<j0kHMp0|is both 
a|f/tokio-1.48.0/src/runtime/context/current.rs|.......A@Z2<0|brary/std/src/io
|/mode](1k8|/rust/deps/addr2line-0.25.1/src/unit.rs|...e](1k0|/rust/deps/rustc
6|-demangle-0.1.26/src/legacy.rs|..A=Rj%5|ibrary/alloc/src/sync.rs|...0dFA^|rgo/
4|.13/src/util/search.rs|A=Rkf0|cargo/registry/src/index.crates.io-1949c
c|f8c6b5b557f/regex-automata-0.4.13/src/meta/wrappers.rs|........A=Rjq0|heap usa
d|dex.crates.io-1949cf8c6b5b557f/dotenv-0.15.0/src/parse.rs|..........A@ZV0|argo
2|src/config.rsA@:1>ZYDLtZYn*[0|argo/registry/src/index.crates.io-1949cf
a|8c6b5b557f/tracing-core-0.1.35/src/callsite.rs|......A=Rkf0|cargo/registry/s
2|a/remapper.rsA@XZu0|ompiling DFA with total patterns in all match st
3|/src/util/empty.rsA=Rkf0|rustup/toolchains/nightly-2025-11-28-x86_64-
e|unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cell.rs|.........e](1k
5|x-0.8.8/src/hir/literal.rs|.A=Rkf0|cargo/registry/src/index.crates.io-1
a|949cf8c6b5b557f/regex-syntax-0.8.8/src/debug.rs|.....e](1k0|library/std/src/
9|sys/pal/unix/stack_overflow/thread_info.rs|.....A=Rkf0|cargo/registry/src/i
3|gistry/sharded.rs|A@ZVf0|ustup/toolchains/nightly-2025-11-28-x86_64-u
7|race-0.3.76/src/symbolize/gimli.rs|...A=Rkf0|cargo/registry/src/index.cra
9|/regex-automata-0.4.13/src/meta/error.rs|.......0dFA^0|rgo/registry/src/ind
2|y/generic.rs|0bMvU5|rary/std/src/io/stdio.rs|...01r*P1|nicked aBrFv2iTSUU3uP%4
6|/slice/sort/shared/smallsort.rs|.e](1k7hmVf3|alid field filter:AYLtU0dFA^|rgo/
a|miniz_oxide-0.8.9/src/inflate/output_buffer.rs|......A=Ri$0|invalid needles 
a|5b557f/aho-corasick-1.1.4/src/util/prefilter.rs|.....e](1k0|~cargo/registry/
1|/slot.rs0dF%h0|stup/toolchains/nightly-2025-11-28-x86_64-unknown-li
c|nux-gnu/lib/rustlib/src/rust/library/std/src/io/mod.rs|........A=Rkf0|rustup/t
7|b/src/rust/library/alloc/src/str.rs|..e](1k0|~cargo/registry/src/index.cr
d|ates.io-1949cf8c6b5b557f/sharded-slab-0.1.7/src/page/mod.rs|........e](1k2oUci
0|/index.crates.io-1949cf8c6b5b557f/gimli-0.32.3/src/read/index.rs
a|lib/rustlib/src/rust/library/alloc/src/sync.rs|......A=RjD0|compiling DFA wi
2|til/search.rsA@ZV00|argo/registry/src/index.crates.io-1949cf8c6b5b55
8|7f/memchr-2.7.6/src/arch/all/twoway.rs|....A=Rkf0|cargo/registry/src/index
c|u/lib/rustlib/src/rust/library/std/src/thread/mod.rs|..........0bMvU0|rary/std
9|/src/sys/thread_local/destructors/list.rs|......A@VOS1|yte indeCP:^q0| is out 
c|8c6b5b557f/backtrace-0.3.76/src/backtrace/libunwind.rs|........A=Ri<0|state le
d|8c6b5b557f/regex-automata-0.4.13/src/meta/reverse_inner.rs|.........A=Rkf|rust
a|stlib/src/rust/library/alloc/src/raw_vec/mod.rs|.....e](1k3@Za>1|RT(ALL):aIX0+
8|0a39/library/core/src/ops/function.rs|.....A@W>l0|ust/deps/miniz_oxide-0.8
6|.9/src/inflate/output_buffer.rs|.e](1k0|library/core/src/num/flt2dec/mod
e|s.io-1949cf8c6b5b557f/regex-automata-0.4.13/src/meta/stopat.rs|..........A=Ri}
6|ax-0.8.8/src/hir/interval.rs|....054>]|v/nuy?%8/0|ibrary/std/src/sys/sync/
2|rwlock/futex.e](1k6|library/std/src/thread/mod.rs|...A@Z2<0|brary/core/src/n
6|itional terms or conditions.|....3t1x(0|nt crates/jeb/src/bin/jeb.rs:54j
0|ebmessagecrates/jeb/src/bin/jeb.rsassertion failed: self.replace
2|c/tid.rs:163:iX&z%0|note: we were already unwinding due to a previou
0|c/index.crates.io-1949cf8c6b5b557f/sharded-slab-0.1.7/src/tid.rs
d|ry/sharded.rs/!\ Tried to register the null callsite /!\|...........3lSqY|is s
jJy3C.rsBF|l$ H%ikT!nqRddawvXpmkrjVg22lQ=p*qEkO66$k[xc$ASX<?@.ekZ5kw0?5dYrRk[sLc
