# Z855 Fuzz Testing

Fuzz tests for z855 encoder/decoder using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Setup

Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

Run a specific fuzz target:
```bash
cargo fuzz run fuzz_roundtrip
cargo fuzz run fuzz_decoder_robustness
cargo fuzz run fuzz_position_invariance
```

Run with limited iterations (for CI):
```bash
cargo fuzz run fuzz_roundtrip -- -runs=100000
```

## Fuzz Targets

### `fuzz_roundtrip`
Tests the fundamental roundtrip property: `decode(encode(data)) == data`

Also verifies length bounds from DESIGN-CONSTRAINTS.md.

### `fuzz_decoder_robustness`
Tests that the decoder never panics on any UTF-8 input.

This is critical for the "liberal decoder" design principle - we must accept any input gracefully.

### `fuzz_position_invariance`
Tests the core position invariance requirement (§3 P1 from DESIGN-CONSTRAINTS.md):

Z85-encoded blocks must appear at the same character positions as standard Z85 encoding.

## Corpus & Artifacts

- `corpus/`: Interesting inputs discovered during fuzzing (git-ignored)
- `artifacts/`: Crash-inducing inputs (git-ignored, should be investigated if found)

## Integration with Property Tests

The fuzz targets complement our proptest-based property tests in `src/proptest.rs` and `src/proptest_priority1.rs`:

- **Fuzz tests**: Good for finding edge cases via random exploration
- **Property tests**: Good for systematic coverage of specific scenarios

Both test the same invariants but with different strategies.
