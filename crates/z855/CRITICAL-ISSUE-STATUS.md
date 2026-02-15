# CRITICAL ISSUE: bytesRemaining Fix Breaks Error Validation

## Current Status

### ✅ What Works
- **Gap tests:** All 1200 tests pass (TypeScript, min.mjs)
- **Normal decoding:** Round-trip encode→decode works
- **Rust unit tests:** 55/55 passing

### ❌ What's Broken
- **Error validation:** ALL 6+ error test cases now incorrectly pass
  - 20-invalid-char
  - 21-invalid-space
  - 22-overflow-5chars
  - 23-overflow-partial
  - 24-invalid-single
  - 56-comma-incomplete-passthrough

- These tests should **REJECT** malformed input but now **ACCEPT** it

## The Problem

### Root Cause
Padding calculation depends on `bytesRemaining` (encoder knows this from context), but decoder can't know it without context.

### Failed "Solution"
Using `z855InputLength(input.length)` to estimate `totalBytesToDecode`:
- ✅ Works for **valid** inputs (makes padding calculation succeed)
- ❌ Breaks for **invalid** inputs (incorrect estimate hides errors)

## The Core Issue

The encoder uses **context-dependent** padding:
```typescript
// Encoder knows bytesRemaining from position in input
availableChars = z855OutputLength(bytesRemaining) - z855OutputLength(bytesAfter)
```

The decoder can't know bytesRemaining without decoding first (chicken-and-egg problem).

## Possible Solutions

### Option 1: Change Encoder (Recommended)
Make encoder use **context-independent** padding formula:
```typescript
// Use isolated formula (only depends on rawLen)
availableChars = z855OutputLength(rawLen) - z855OutputLength(rawLen - 8)
```

**Pros:**
- Decoder can calculate without context
- No error validation regression
- Simpler and more predictable

**Cons:**
- Changes encoder behavior
- Need to re-encode all test files
- May not maintain length invariant in all cases

### Option 2: Encode Context in Prefix
Add `bytesRemaining` or `availableChars` to the escape prefix.

**Pros:**
- Decoder has exact information
- No guessing needed

**Cons:**
- Format change (bigger prefix)
- More complex encoding/decoding

### Option 3: Revert bytesRemaining Fix
Go back to previous state, accept that some cases don't work.

**Cons:**
- Test 119-long-escape-9bytes-then-unsafe fails
- Can't decode files encoder produces

## Recommendation

**Change the encoder to use context-independent padding.**

The current encoder's context-dependent padding is fundamentally incompatible with forward-only decoding. The decoder MUST be able to calculate padding without knowing future context.

The fix: modify encoder at lines 1488-1494 (z855.ts) to use:
```typescript
const availableChars = z855OutputLength(rawLen) - z855OutputLength(rawLen - 8);
```

Instead of:
```typescript
const availableChars = z855OutputLength(bytesRemaining) - z855OutputLength(bytesAfter);
```

## Next Steps

1. Revert decoder bytesRemaining changes
2. Modify encoder to use context-independent padding
3. Re-encode all test files
4. Verify all tests pass (including error cases)
5. Apply same fix to Rust and min.mjs encoders

---

**User's requirement:** "We need to make sure that all are consistent, including rust. WE shoudl be tesitng all three all the time and fixing it i fit's not working fix it fix it"

We need to fix the ENCODER, not just the decoder!
