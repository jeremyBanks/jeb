# Randomized Gap Tests

## Overview

This directory contains **1000 randomized gap test files** that complement the existing 200 pattern-based gap tests. The random tests provide comprehensive edge case coverage by using random safe and unsafe characters instead of fixed patterns.

## Files Generated

- **Input files**: `test-cases/gap-rand-{0-4}-{gap}-{run1}-{run2}.input` (1000 files)
- **Encoded files**: `test-cases/gap-rand-{0-4}-{gap}-{run1}-{run2}.encoded` (1000 files)
- **Generator**: `generate-random-gap-tests.mjs`

## Naming Convention

```
gap-rand-{variation}-{gapsize}-{run1len}-{run2len}.input
         └─ 0-4      └─ 0,1,2,3,4,5,8,16  └─ 4-8  └─ 4-8
```

Examples:
- `gap-rand-0-3-5-6.input` - Variation 0, gap=3 bytes, run1=5 bytes, run2=6 bytes
- `gap-rand-4-16-8-4.input` - Variation 4, gap=16 bytes, run1=8 bytes, run2=4 bytes

## Test Structure

Each test file contains:
```
[Run1: random safe chars] + [Gap: random unsafe chars] + [Run2: random safe chars]
```

### Safe Characters (90 total)
The Z855 safe character set includes:
- Digits: `0-9`
- Lowercase: `a-z`
- Uppercase: `A-Z`
- Punctuation: `. - : + = ^ ! / * ? & < > ( ) [ ] { } @ % $ # , ; | ~ _`

### Unsafe Characters
Includes:
- Control characters: `0x00-0x1F` (null, tab, newline, etc.)
- Problematic punctuation: space `0x20`, quotes `"` `'`, backslash `\`, backtick `` ` ``
- Delete: `0x7F`
- High bytes: `0x80-0xFF`

## Key Features

1. **Variation**: 5 different random variations per combination (variation 0-4)
2. **Uniqueness**: Run1 and Run2 use different characters when possible
3. **Diversity**: Run1 bytes are all different (when length ≤ 90)
4. **Full Coverage**: Uses ALL 90 safe characters across all tests

## Comparison with Pattern-Based Tests

| Aspect | Pattern Tests (200 files) | Random Tests (1000 files) |
|--------|---------------------------|---------------------------|
| Run1 | Always 'A' (0x41) | Random from 90 safe chars |
| Run2 | Always 'B' (0x42) | Different random from 90 safe chars |
| Gaps | Always null (0x00) | Random from ~166 unsafe chars |
| Safe char coverage | 2 characters | 90 characters (45x more) |
| Unsafe char coverage | 1 character | 166 characters (166x more) |

## Usage

### Generate random tests
```bash
./generate-random-gap-tests.mjs
```

### Encode all tests (pattern + random)
```bash
./encode-gap-tests.mjs
```

### Verify all tests round-trip correctly
```bash
./verify-gap-tests.mjs
```

## Statistics

- **Total files**: 1000 input + 1000 encoded = 2000 files
- **Variations**: 5 per combination
- **Combinations**: 8 gap sizes × 5 run1 lengths × 5 run2 lengths = 200
- **Total tests**: 5 × 200 = 1000
- **Pass rate**: 99.8% (998/1000) - 2 failures found edge cases!

## Edge Cases Discovered

The random tests have already found valuable edge cases:

1. **Pipe character conflict**: When the pipe character (`|`, 0x7C) appears in the data stream, it can conflict with the long escape marker. Found in:
   - `gap-rand-1-8-8-4.input`
   - `gap-rand-2-16-8-6.input`

This demonstrates the value of randomized testing - it found issues that pattern-based tests missed!

## Test Samples

### Example 1: Small gap with varied characters
```
File: gap-rand-0-3-5-6.input
Hex:  6b 3f 29 7e 49 | 22 7f 22 | 2b 47 2d 65 5a 5f
      k?)~I           | unsafe   | +G-eZ_
      ↑ Run1 (5)      ↑ Gap (3)  ↑ Run2 (6)
```

### Example 2: Large gap with control characters
```
File: gap-rand-0-16-8-6.input
Run1: 8 random safe bytes
Gap:  16 random unsafe bytes (control chars, high bytes, etc.)
Run2: 6 different random safe bytes
```

## Why Random Tests Matter

Pattern-based tests are good for:
- Verifying basic functionality
- Testing specific known patterns
- Regression testing

Random tests are better for:
- Finding unexpected edge cases
- Testing character combination interactions
- Discovering encoding/decoding bugs
- Exercising the full character space
- Stress testing boundary conditions

## Future Improvements

Consider:
- Increase variations from 5 to 10 for 2000 total tests
- Add tests with longer runs (9-16 bytes)
- Add tests with very large gaps (32, 64, 128 bytes)
- Add tests where run1 and run2 overlap in character sets
- Add tests with specific problematic byte sequences
