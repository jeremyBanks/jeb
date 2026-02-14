a fever dream

---

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

Copyright Jeremy Banks and contributors.

Licensed under either of:

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
