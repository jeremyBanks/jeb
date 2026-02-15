# Pipe Character Bug Analysis

## Issue Discovered

The randomized gap tests found **2 failing test cases** involving the pipe character `|` (0x7C):
- `gap-rand-1-8-8-4.input`
- `gap-rand-2-16-8-6.input`

## Root Cause

**Pipe character is BOTH:**
1. A safe character that can appear in raw passthrough data
2. The escape marker for long (8+ byte) passthrough

This creates ambiguity in certain edge cases.

## Failing Test Case Analysis

### Test: gap-rand-1-8-8-4

**Input bytes:**
```
Positions 0-7:   77 76 5b 31 5d 4a 63 4f  |wv[1]JcO|  (8 safe bytes)
Position 8:      c2                        (unsafe)
Positions 9-15:  df 22 22 20 7f 20 7f     (unsafe)
Positions 16-19: 46 7c 34 54              |F|4T|      (4 safe bytes, note 0x7C at pos 17)
```

**Canonical encoding:**
```
8|wv[1]JcO.RVKYaB+I-,F|4T
```

**Breakdown:**
1. `8|` - Long escape for 8 bytes
2. `wv[1]JcO` - 8 raw safe bytes (positions 0-7)
3. `.RVKYaB+I-` - Encoded portion (position 8-15)
4. `,F|4T` - Comma escape for 4 safe bytes (positions 16-19)

**The Problem:**

Position 17 contains `0x7C` which is the pipe character `|`. When this appears in a comma escape immediately after a long escape, the decoder gets confused:

```
8|wv[1]JcO.RVKYaB+I-,F|4T
                      ^
                      Decoder might interpret this | as the
                      terminator for the long escape instead
                      of as data in the comma escape!
```

## Decoder Logic Issue

**From z855.ts lines 676-692:**

After reading raw bytes from a long escape, the decoder tries to skip padding and find a final `|` terminator:

```typescript
// Now we need to skip the remaining padding (. characters) and final |
// Padding format: [. chars][|] or just [|] if no padding needed
// Or no padding at all for exact fit (8 bytes)
while (inIdx < input.length) {
  const nextChar = input.charCodeAt(inIdx);
  if (nextChar === RAW_ESCAPE_PADDING) {
    // Skip padding
    inIdx += 1;
  } else if (isLongEscape(nextChar)) {
    // Final | (aesthetic terminator)
    inIdx += 1;
    break;
  } else {
    // End of padding section, continue normal decoding
    break;
  }
}
```

**From encoder (lines 1548-1551):**

```typescript
// Add remaining padding: all dots, no final |
for (let i = 0; i < paddingAfter; i++) {
  output.push(String.fromCharCode(RAW_ESCAPE_PADDING));
}
```

**The Issue:**

1. **Encoder**: Outputs padding dots but NO final `|` terminator (comment says "no final |")
2. **Decoder**: Looks for optional padding dots followed by optional final `|`
3. **When a pipe appears in subsequent data**: The decoder may incorrectly consume it as a terminator

## Specific Failure Scenario

```
Encoding: 8|wv[1]JcO.RVKYaB+I-,F|4T
                                  ^
After long escape:                |
- Decoder reads 8 bytes: wv[1]JcO
- Decoder at position: .RVKYaB+I-,F|4T
- Decoder skips to find |
- Decoder finds | at position shown above
- Decoder thinks this is the terminator!
- Decoder skips it and continues
- Now decoder tries to parse 4T as next escape
- ERROR: Invalid decoding
```

## Why Pattern Tests Didn't Catch This

**Pattern-based tests use:**
- First run: All A's (0x41)
- Second run: All B's (0x42)
- Gaps: All zeros (0x00)

**None of these bytes are pipe (0x7C)!**

The pattern tests NEVER exercise the case where:
- A long escape is followed by
- Another escape that contains
- A pipe character in its raw data

## Why Randomized Tests Caught It

**Randomized tests use:**
- Random safe characters from the FULL safe set (90 characters including pipe)
- Random unsafe characters from the full unsafe set (166 characters)

**This gives 85x more character variety** and immediately hit the edge case where pipe appears in data following a long escape.

## Potential Solutions

### Option 1: Remove Pipe from Safe Set

**Pros:**
- Eliminates ambiguity
- Simplest fix

**Cons:**
- Changes the safe character set
- Pipe is a useful character for passthroughs
- Breaking change to specification

### Option 2: Always Output Final Pipe Terminator

**Pros:**
- Decoder logic already expects it
- Clear termination marker

**Cons:**
- Increases output size by 1 char per long escape
- Current encoder comment says "no final |"

### Option 3: Make Decoder More Robust

**Pros:**
- No spec changes needed
- Maintains backward compatibility

**Cons:**
- More complex decoder logic
- Need to carefully track state

### Option 4: Use Different Terminator

**Pros:**
- Could use a non-safe character as terminator
- Eliminates ambiguity

**Cons:**
- Breaking change to specification
- Requires choosing a new terminator character

## Recommendation

**Option 3: Fix the decoder**

The issue is that after reading `rawLen` bytes, the decoder should NOT look for a pipe terminator if there's no padding. The decoder should:

1. Read exactly `rawLen` bytes (already correct)
2. Skip padding dots if present
3. ONLY look for closing `|` if padding was present
4. Otherwise, immediately continue normal decoding

The encoder already doesn't output a final `|` (comment says "no final |"), so the decoder shouldn't expect one in all cases.

## Test Cases That Expose This Bug

Out of 1000 randomized tests, only 2 failed (~0.2% failure rate). Both involve:
- A long (8+) byte passthrough escape
- Followed by another escape (likely `,` for 4 bytes)
- Where the raw data contains a pipe character (0x7C)

**Specific failing tests:**
1. `gap-rand-1-8-8-4` - gap of 1, run1=8, run2=4
2. `gap-rand-2-16-8-6` - gap of 16, run1=8, run2=6

Both have 8-byte first runs (triggering long escape) and the second run contains a pipe character in the raw data.

## Impact

**Low severity but important to fix:**
- Affects only 0.2% of random inputs (2/1000 tests)
- Only when specific character (pipe) appears in specific positions
- But when it happens, decoding fails completely
- Pattern-based tests would NEVER catch this

This demonstrates the value of randomized testing with full character coverage!
