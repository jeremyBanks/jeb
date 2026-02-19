# Padding Tests Generation Summary

## Overview

Successfully generated **112 randomized padding tests** as specified in PADDING-TEST-REQUIREMENTS.md.

These tests expose a critical bug in the Z855 decoder: it checks padding **content** instead of skipping padding based on **position** calculation.

## Test Matrix

- **Raw lengths**: 8, 16, 24, 32 bytes (4 values)
- **Offsets**: 0, 1, 2, 4 (4 values)
- **Padding patterns**: dots, pipes, zeros, safe, unsafe, mixed, escapes (7 patterns)
- **Total**: 4 × 4 × 7 = **112 test cases**

Each test case generates 2 files:
- `.input` - raw bytes to encode
- `.encoded` - Z855 long escape format with custom padding

## Test Distribution

### By Raw Length
- 8 bytes: 28 tests (4 offsets × 7 patterns)
- 16 bytes: 28 tests
- 24 bytes: 28 tests
- 32 bytes: 28 tests

### By Offset
- Offset 0: 28 tests (4 lengths × 7 patterns)
- Offset 1: 28 tests
- Offset 2: 28 tests
- Offset 4: 28 tests

### By Padding Pattern
- dots: 16 tests (4 lengths × 4 offsets) - canonical padding
- pipes: 16 tests - **exposes the bug!**
- zeros: 16 tests - null bytes in padding
- safe: 16 tests - random safe chars
- unsafe: 16 tests - random unsafe chars (control chars, high bytes)
- mixed: 16 tests - completely random bytes
- escapes: 16 tests - escape chars (,;|~_)

## Verification Results

✅ **All 112 tests generated successfully**

Running `verify-padding-tests.mjs`:
- **16 tests PASS** (dots pattern) - canonical padding works
- **96 tests FAIL** (all other patterns) - **exposes decoder bug**

### Pattern Results
| Pattern  | Status       | Pass/Total | Expected |
|----------|--------------|------------|----------|
| dots     | ✅ ALL PASS  | 16/16      | PASS     |
| escapes  | ❌ ALL FAIL  | 0/16       | FAIL     |
| mixed    | ❌ ALL FAIL  | 0/16       | FAIL     |
| pipes    | ❌ ALL FAIL  | 0/16       | FAIL     |
| safe     | ❌ ALL FAIL  | 0/16       | FAIL     |
| unsafe   | ❌ ALL FAIL  | 0/16       | FAIL     |
| zeros    | ❌ ALL FAIL  | 0/16       | FAIL     |

## Example Test Cases

### Test 1: padding-8-0-dots (baseline - should pass)
```
Raw length: 8 bytes
Offset: 0
Pattern: dots (canonical)
Encoded: 8|qwh_4s*-..........
```
Structure:
- `8` = length prefix
- `|` = separator
- `qwh_4s*-` = 8 raw bytes
- `..........` = 10 padding dots

**Status**: ✅ PASS

### Test 2: padding-8-0-pipes (exposes bug)
```
Raw length: 8 bytes
Offset: 0
Pattern: pipes
Encoded: 8|^Bg]S;p@||||||||||
```
Structure:
- `8` = length prefix
- `|` = separator
- `^Bg]S;p@` = 8 raw bytes
- `||||||||||` = 10 padding pipes

**Status**: ❌ FAIL - "no prefix digits before |"

The decoder sees the pipe chars in padding and gets confused!

### Test 3: padding-16-2-safe (offset + random safe padding)
```
Raw length: 16 bytes
Offset: 2
Pattern: safe (random safe chars)
Encoded: 2g|Gq]mfpM3^1]&i9b@gQTr>4^+-
```
Structure:
- `2` = offset prefix (offset=2)
- `g` = length prefix (16 bytes)
- `|` = separator
- `Gq` = 2 padding bytes (before raw data)
- `]mfpM3^1]&i9b@gQ` = 16 raw bytes
- `Tr>4^+-` = 7 padding bytes (after raw data)

Padding calculation:
- Total padding = ceil(16×5/4) - ceil((16-8)×5/4) = 20 - 10 = 10
- Offset prefix length = 1
- Padding before = 2 (offset)
- Padding after = 10 - 1 - 2 = 7

**Status**: ❌ FAIL - "expected padding dot for offset"

## The Bug

The current decoder (z855.ts lines 676-692) checks padding content:

```typescript
while (inIdx < input.length) {
  const nextChar = input.charCodeAt(inIdx);
  if (nextChar === RAW_ESCAPE_PADDING) {  // ❌ Assumes . is padding
    inIdx += 1;
  } else if (isLongEscape(nextChar)) {     // ❌ Assumes | terminates
    inIdx += 1;
    break;
  } else {
    break;
  }
}
```

**This is WRONG!**

The decoder should:
1. Calculate padding amount from offset and length
2. Skip exactly that many positions
3. **Ignore padding content completely**

## Files Generated

Total: 224 files (112 test cases × 2 files each)

Location: `test-cases/padding-*.{input,encoded}`

Naming convention:
```
padding-<rawlen>-<offset>-<pattern>.input
padding-<rawlen>-<offset>-<pattern>.encoded
```

Examples:
```
test-cases/padding-8-0-dots.input
test-cases/padding-8-0-dots.encoded
test-cases/padding-8-0-pipes.input
test-cases/padding-8-0-pipes.encoded
test-cases/padding-32-4-mixed.input
test-cases/padding-32-4-mixed.encoded
```

## Next Steps

1. ✅ Generate 112 randomized padding tests - **COMPLETE**
2. ✅ Verify tests fail (expose decoder bug) - **CONFIRMED**
3. ⏳ Commit failing tests (document the bug)
4. ⏳ Fix decoder (position-based padding skip)
5. ⏳ Verify all tests pass

## Scripts

### Generation Script
`generate-padding-tests.mjs` - Creates all 112 test cases
- Generates random raw bytes (safe chars)
- Calculates padding positions
- Creates long escape format with custom padding patterns
- Writes .input and .encoded files

### Verification Script
`verify-padding-tests.mjs` - Verifies roundtrip decode
- Reads all padding-*.input files
- Decodes the corresponding .encoded files
- Compares decoded output with original input
- Reports results by pattern

### Quick Test Script
`test-single-decode.mjs` - Tests a single encoded file
- Shows detailed structure breakdown
- Demonstrates the decoder error

## Conclusion

✅ **All 112 padding tests successfully generated and verified**

The tests correctly expose the decoder bug:
- Canonical padding (dots) works
- All other padding patterns fail
- This proves the decoder checks content instead of position

Ready for next steps: commit tests, fix decoder, verify all pass.
