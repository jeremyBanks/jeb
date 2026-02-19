# Task: Complete z855-reference.ts

## Context

This is a z855 codec implementation. z855 is an extension of Z85 (ZeroMQ RFC 32) that:
1. Supports arbitrary-length input (Z85 only supports multiples of 4 bytes)
2. Adds "raw passthrough" escapes for readable bytes

## Your Goal

Complete `z855-reference.ts` into a fully working, well-tested implementation.
**Commit extremely often — after every meaningful change.**
**Never amend, rebase, or force-push.**

## Key Design Decisions (the sketch is authoritative)

### Escape byte assignments (NEW — different from production z855.ts):
- `_` (0x5F) = escape 4 raw bytes
- `,` (0x2C) = escape 5 raw bytes  
- `~` (0x7E) = escape 6 raw bytes
- `;` (0x3B) = escape 7 raw bytes
- `|` (0x7C) = long escape (8+ raw bytes, length-prefixed)
- `0|` = rest-of-input raw (no padding, can't be used in concatenatable mode)

### Long escape prefix encoding (base-42, self-terminating):
- Scan BACKWARDS from `|`
- Digit value 0–41: terminal digit (ends the number)
- Digit value 42–83: continuation digit (value − 42, more digits follow)
- Z85 alphabet is used for digit characters
- After reading length: if length > 15, also read offset the same way (how many dot-padding bytes precede raw data)
- If length 8–15: no offset prefix possible, effective offset = 0
- Size ranges and padding:
  - 8–11 bytes: no padding needed (output length = ceil(N*5/4) exactly)
  - 12–15: 1 padding byte (`.`) at the end
  - 16+: variable padding, offset encoded in prefix

### Z85 alphabet:
`0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#`

### Safe bytes for passthrough:
Z85 alphabet + `_,~;|` (the 5 escape characters themselves)

### Partial blocks (1–3 trailing bytes):
- Canonical mode: N bytes → N+1 chars (no padding)
- Concatenatable mode: N bytes → hash-padded to 5 chars, with `#` chars prepended

### Non-aligned passthrough:
When a run of safe bytes doesn't start at a block boundary, we can still pass them through.
The encoder emits partial Z85 for the before-block, then the escape, then the raw bytes.
The decoder must recover the before-block using the `findAllValues` / `findMinValue` solver
(already implemented in the sketch — use it!).

**For `,`/`_`/`~`/`;` escapes:**
- `,` (5-byte): P chars before escape, then escape, then 5 raw bytes, then (5-P) chars after
  - P chars uniquely determine the before-block given the known bytes (canonical minimum)
- `_` (4-byte): similar but P+1 chars before escape (extra char eliminates ambiguity)
- `~` (6-byte): P+1 chars before escape  
- `;` (7-byte): P+1 chars before escape

Wait, let me clarify from the production implementation context:
- 4-byte escape (`,` in production, `_` in the): P chars before, uses canonical minimum
- 5/6/7-byte escapes: (P+1) chars before, no canonical minimum needed (extra char disambiguates)

## Files to Study

1. `z855-reference.ts` — the incomplete sketch (your primary target)
2. `z855-readable.ts` — my completed readable implementation (different escape ordering but same concepts)
3. `z855.ts` — production implementation (complex, reference only)
4. `test-cases/` — 1416 test cases (but these use the OLD escape ordering!)

## Test Strategy

The test-cases directory uses the OLD escape ordering (`,`=4, `;`=5, `_`=6, `~`=7).
the new ordering is (`_`=4, `,`=5, `~`=6, `;`=7).

So you CANNOT directly use the existing test-cases to validate the encoder output format.
However, you CAN use them to validate the **decoder** for old-format encoded strings if needed.

Instead, write your own tests:
1. Encode with the reference encoder → decode with the reference decoder → check roundtrip
2. Test specific cases from the sketch/spec
3. Cross-check against `z855-readable.ts` for the algorithmic logic (same logic, different escape chars)

## Implementation Plan

### Phase 1: Fix known bugs and complete the scaffolding
- Fix `valueToDigits` return type (returns number[] but used as Uint8Array)
- Fix `blockDigits` usage (digit indices vs byte values)  
- Fix the `0|` rest-of-input bug: `original.subarray(inputOffset, safeLength)` should be `original.subarray(inputOffset, inputOffset + safeLength)`
- Fix concatenatable partial block: hash padding goes at FRONT, not overwriting digits
- Fix `encodedOffset += safeLength - inputOffset` bug

### Phase 2: Implement the core encoder logic
The sketch has `throw new Error("not implemented")` for the main passthrough logic.
Fill this in by:
1. Given `safeLength` (total safe bytes in run) and position in block, pick the best escape form
2. Use `findAllValues`/`findMinValue` for non-aligned passthrough canonicality checks
3. Handle all escape forms: `_` (4), `,` (5), `~` (6), `;` (7), long `|` (8+), `0|` (rest-of-input)

### Phase 3: Implement the decoder
Complete `decode()`. Follow the same state-machine approach as `z855-readable.ts` but adapted
for the new escape ordering.

### Phase 4: Write tests
Create `z855-reference-test.ts` with comprehensive tests:
- Empty input
- All lengths 0–20 (roundtrip)  
- Known specific values (zeros, max, etc.)
- All escape forms triggered explicitly
- Concatenatable mode
- Error cases (overflow, invalid chars, incomplete escapes)

### Phase 5: Wire up CLI and run it
```bash
echo "hello world test 1234" | deno run z855-reference.ts encode | deno run z855-reference.ts decode
```

## Key Helper Already Implemented

`findAllValues(chars, bytes)` and `findMinValue(chars, bytes)` are complete and correct.
Use `findMinValue` for the canonical minimum in non-aligned passthrough.

Example usage for non-aligned 4-byte (5 raw bytes with `_` escape, P chars before):
```typescript
// Before block: we know P Z85 chars and (4-P) bytes from passthrough
const chars = [knownChar0, null, null, null, null]; // P known chars
const bytes = [null, null, passBytes[0], passBytes[1], passBytes[2]]; // 4-P known bytes (wrong indices, fix per P)
const canonical = findMinValue(chars, bytes);
// If actual before-block value !== canonical, can't use non-aligned passthrough here
```

## Running Tests

```bash
cd /Users/matte/jeb/crates/z855
deno test z855-reference-test.ts --allow-read
deno run --allow-read z855-reference.ts encode < /dev/null
```

## Done Criteria

- [ ] `encode` and `decode` are fully implemented
- [ ] Roundtrip works for all byte lengths 0–64
- [ ] All escape forms (_, , ~, ;, |, 0|) are exercised by tests
- [ ] Concatenatable mode works
- [ ] Tests pass with `deno test`
- [ ] CLI works: `echo "test" | deno run z855-reference.ts encode | deno run z855-reference.ts decode`
- [ ] Committed with frequent intermediate commits

When completely finished, run:
openclaw system event --text "Done: z855-reference.ts fully implemented and tested" --mode now
