# Padding Test Requirements - Critical Gap

## The Problem

**CRITICAL BUG DISCOVERED:** The decoder assumes `|` can be used as a padding terminator, but this violates the spec!

**The decoder MUST:**
- Skip padding based on POSITION only (calculated from offset/length)
- Ignore padding CONTENT completely
- Work with ANY bytes in padding positions

**Current tests FAIL to verify this** because they only use the canonical encoder's deterministic padding (all dots `.`).

## Why This Matters

The pipe character bug (0x7C) exposed that:
1. Decoder looks for `|` to find end of padding
2. If `|` appears in subsequent DATA, decoder gets confused
3. This means decoder is checking padding CONTENT, not just skipping POSITIONS

**But the spec says:** Padding bytes can be ANYTHING - the decoder should skip exactly N bytes based on the calculated padding amount, regardless of what those bytes are!

## Current Decoder Bug (z855.ts lines 676-692)

```typescript
// Now we need to skip the remaining padding (. characters) and final |
// Padding format: [. chars][|] or just [|] if no padding needed
// Or no padding at all for exact fit (8 bytes)
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

**This is WRONG!** Should calculate and skip exact positions.

## What We Need: Randomized Padding Tests

### Test Strategy

1. **Generate valid long escapes** with calculated padding
2. **Replace padding bytes** with random content
3. **Verify decoder works** (position-based skip)

### Test Matrix

For each combination:
- **Raw lengths:** 8, 16, 24, 32 bytes
- **Offsets:** 0, 1, 2, 4
- **Padding patterns:**
  - All dots `.` (baseline - should work)
  - All pipes `|` (will expose bug!)
  - All zeros `0x00`
  - Random safe chars
  - Random unsafe chars
  - Mixed random
  - Escape chars `,;_~|`

**Total:** 4 lengths × 4 offsets × 7 patterns = 112 test cases

### File Naming

```
test-cases/padding-<rawlen>-<offset>-<pattern>.input
test-cases/padding-<rawlen>-<offset>-<pattern>.encoded
```

Patterns:
- `dots` - all `.` (canonical)
- `pipes` - all `|`
- `zeros` - all `0x00`
- `safe` - random safe chars
- `unsafe` - random unsafe chars
- `mixed` - random bytes
- `escapes` - escape chars `,;_~|`

## Success Criteria

After fixing decoder:

✅ Decoder calculates padding positions from offset/length
✅ Decoder skips exactly that many positions
✅ Decoder does NOT check padding content
✅ All 112 tests pass

## Next Steps

1. **Generate 112 randomized padding tests** ← DO THIS FIRST
2. **Commit failing tests** (document the bug)
3. **Fix decoder** (position-based skip)
4. **Verify all tests pass**
