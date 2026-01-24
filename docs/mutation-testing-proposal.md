# Mutation Testing Proposal for jeb

## Overview

This document proposes integrating [cargo-mutants](https://mutants.rs/) into our
testing workflow to identify gaps in test coverage and improve overall code
quality.

## What is Mutation Testing?

Mutation testing works by introducing small bugs (mutations) into the code and
checking whether the test suite catches them. If a mutation survives (tests
still pass), it indicates a gap in test coverage.

## Initial Findings

Running `cargo mutants` on `jeb-common` revealed significant gaps in test
coverage for the bijection functions:

### scatter_square module results (partial):

| Result | Count | Description |
|--------|-------|-------------|
| MISSED | ~23 | Tests don't catch these bugs |
| TIMEOUT | ~22 | Mutations cause infinite loops |
| CAUGHT | 0 | Tests successfully detect bugs |

### Key observations:

1. **Helper functions are undertested**: Internal functions like `mix64()`,
   `key_for_layer()`, and `next_pow2()` have mutations that go completely
   undetected by the current test suite.

2. **Property tests verify behavior, not implementation**: The proptest-based
   tests verify high-level properties (roundtrip correctness, bijection
   guarantees) but don't test internal implementation details.

3. **Some mutations cause infinite loops**: Many mutations in loop conditions or
   iterative functions cause tests to hang rather than fail, indicating these
   code paths lack bounds checking in tests.

## Recommendations

### Short-term improvements

1. **Add unit tests for helper functions**:
   - `mix64()` - verify mixing properties
   - `next_pow2()` - verify power-of-two rounding
   - `isqrt_u64()` - verify integer square root correctness
   - `key_for_layer()` - verify key generation

2. **Add invariant assertions within bijection tests**:
   ```rust
   // Example: verify intermediate state in scatter_square
   let layer = compute_layer(input);
   assert!(layer <= MAX_LAYER, "layer out of bounds");
   ```

3. **Add timeout guards to property tests**:
   ```rust
   proptest! {
       #![proptest_config(ProptestConfig::with_cases(1000))]
       // ...
   }
   ```

### Medium-term integration

1. **Add mutation testing to CI** (optional, gated):
   ```yaml
   mutation-test:
     runs-on: ubuntu-latest
     if: github.event_name == 'schedule' || contains(github.event.pull_request.labels.*.name, 'mutation-test')
     steps:
       - uses: actions/checkout@v4
       - run: cargo install cargo-mutants
       - run: cargo mutants --package jeb-common -j 4 --timeout 120
   ```

2. **Track mutation score over time**: Aim for catching >80% of mutations in
   critical code paths.

3. **Use mutation testing for new features**: Before merging significant new
   functionality, run mutation testing to verify test adequacy.

### Configuration

Create `mutants.toml` in the workspace root:
```toml
# Exclude test code and generated files
exclude_globs = ["**/tests/**", "**/benches/**", "**/*_test.rs"]

# Increase timeout for complex functions
timeout_multiplier = 2.0

# Skip functions that are known to have equivalent mutants
exclude_re = ["^fmt$", "^display$"]
```

## Running Mutation Tests

```bash
# List all possible mutations
cargo mutants --package jeb-common --list

# Run mutation testing on specific module
cargo mutants --package jeb-common -F "scatter_square" --timeout 60

# Run full mutation testing (slow)
cargo mutants --package jeb-common --timeout 120 --jobs 4
```

## Expected Benefits

1. **Higher confidence in correctness**: Mutations caught = bugs that tests
   would catch in production.

2. **Better test design**: Guides writing tests that verify behavior rather than
   just exercising code paths.

3. **Reduced regression risk**: Ensures tests actually validate the invariants
   we care about.

## References

- [cargo-mutants documentation](https://mutants.rs/)
- [Mutation Testing in Rust](https://blog.frankel.ch/mutation-testing-rust/)
- [GitHub: sourcefrog/cargo-mutants](https://github.com/sourcefrog/cargo-mutants)
