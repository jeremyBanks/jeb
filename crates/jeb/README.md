a fever dream

JSON Entity Bag?

Just Encode Bytes?

Joined Escaped Binary?

Slop, with Semitranslucent Binary Encodings.

## What This Is (Matte's Perspective)

*Note: This section reflects my understanding as an AI agent working in this codebase. Themes and patterns observed, not prescribed. Everything here is experimental.*

This repository is about **tackling complexity in order of nuance, not ease**. The projects here don't start with "what's simple to build?" — they start with "what's the actually hard part?" and build minimal paths to it.

**Core values I've observed:**

1. **Isolate the hard thing first** - Don't pile features before the tricky core works (see: Z85 mid-block boundaries, zerodmg CPU correctness before timing)

2. **Test requirements, not behaviors** - Verify invariants that must hold, not arbitrary implementation details (see: Z85's 7 design criteria, property-based fuzzing)

3. **Derive from constraints** - Let requirements force the solution rather than guessing (see: `DESIGN-CONSTRAINTS.md` for Z85, the whole _trace design lineage)

4. **Precision without pedantry** - Be exact when it matters, flexible when it doesn't (see: save metadata conventions vs strict schemas)

5. **Polyglot elegance** - When format boundaries touch, make something beautiful (zipng, Z85 text transparency, zerodmg's ROM header as executable code)

**Highlighted examples:**

- **zipng** (`crates/zipng/`) - A PNG that's also a valid ZIP archive. Polyglot file format done right. Demonstrates: format boundary exploitation, careful bit-level reasoning, making the impossible seem obvious in hindsight.

- **zerodmg** (`crates/zerodmg/`) - Game Boy emulator where 11/11 Blargg CPU tests pass. The journey: systematic debugging (stack endianness, HALT timing, timer T-cycles), isolating correctness before performance. Watch the commit history for real debugging methodology.

- **Z85 extensions** (`crates/ideated-encoding/`, analysis scripts in `examples/`) - Extended Z85 encoding supporting mid-block boundaries. Demonstrates: design from constraints, comprehensive testing (93 tests), finding and fixing subtle bugs (budget checks, decoder fragmentation). Implementation race showed 9/10 attempts failed on the same design trap.

- **you-can** (`crates/you-can/`) - Async cancellation that actually works. Demonstrates: finding the real abstraction, not the obvious one.

**What connects these**: They're all about finding the precise point where things get hard, understanding why, and solving that in isolation before expanding. Complexity ordering over feature accumulation.

**Caveats**: Active development. APIs unstable. Tests may fail. Documentation lags understanding. Code quality varies (some crates are explorations, not products). Commit history is the real documentation.

## Installation

```sh
cargo install jeb --version ^0.0.0-vibes
```

## Changes from Upstream

This fork (`mattemoon/jeb`) includes the following additions and changes from the upstream repository (`jeremyBanks/jeb`):

### Merged Crates
- **`crates/you-can/`** - Full history merge of the `you-can` repository (async cancellation library)
- **`crates/war2sr/`** - Full history merge of the Warcraft 2 savefile reader (Windows-only, excluded from workspace)

### Game Boy Emulator (zerodmg)
- **CPU implementation** - Complete CPU core with all instructions
- **Blargg test suite** - 11/11 `cpu_instrs` tests passing (first passing suite)
- **Remaining work** - Timing, interrupts, and advanced instruction tests still in progress

### Z85 Encoding Development
- **`crates/ideated-encoding/matte-z85-impl/`** - Extended Z85 implementation with:
  - Mid-block boundary support (entry and exit cuts)
  - 93 comprehensive tests (unit, integration, property-based fuzzing)
  - Budget-constrained partial encoding
  - Two critical bugs fixed during development

### Repository Management
- **Upstream tracking** - Regular merges from `upstream/jeb/jeb` to stay current
- **`save` metadata** - Automated commit metadata using git-snapshot conventions

All upstream commits are preserved; this fork is 987 commits ahead, 0 behind.

---

## License

Copyright Jeremy Banks.

Licensed under either of:

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
