# Gap Test Criteria

## Purpose
Test canonical encoding behavior when two safe character runs are separated by unsafe bytes (gaps).

## Test Matrix

### Run Lengths to Test
- **4 bytes**: "AAAA" + "BBBB" (uses `,` escape)
- **5 bytes**: "AAAAA" + "BBBBB" (uses `;` escape)
- **6 bytes**: "AAAAAA" + "BBBBBB" (uses `_` escape)
- **7 bytes**: "AAAAAAA" + "BBBBBBB" (uses `~` escape)
- **8 bytes**: "AAAAAAAA" + "BBBBBBBB" (uses `|` escape)

### Gap Sizes to Test
- **0 bytes**: Back-to-back runs (e.g., "AAAABBBB")
  - Tests: Can encoder recognize as two separate runs or merges into one?
  - Expected: Should use appropriate escape for combined length if all safe

- **1 byte**: One unsafe byte (0x00)
  - Tests: How is single byte encoded between two escapes?
  - Expected: Both runs visible as raw, 1 byte encoded between

- **2 bytes**: Two unsafe bytes (0x00, 0x00)
  - Tests: Partial block encoding between runs
  - Expected: Both runs visible, 2 bytes encoded

- **3 bytes**: Three unsafe bytes (0x00, 0x00, 0x00)
  - Tests: 3/4 of a Z85 block between runs
  - Expected: Both runs visible, 3 bytes encoded as 4-char Z85

- **4 bytes**: Exactly one Z85 block (0x00, 0x00, 0x00, 0x00)
  - Tests: Full block - does it use `,` escape or pure Z85?
  - Expected: Both runs visible, gap as either "00000" or ",xxxx"

- **5 bytes**: Partial second block (5 zeros)
  - Tests: Just over one block boundary
  - Expected: Both runs visible, 5 bytes encoded

- **8 bytes**: Two complete Z85 blocks (8 zeros)
  - Tests: Multiple blocks between runs
  - Expected: Both runs visible, 8 bytes as "0000000000" or escape

- **16 bytes**: Four complete Z85 blocks (16 zeros)
  - Tests: Long gap behavior
  - Expected: Both runs visible, efficient gap encoding

## Success Criteria

For each test case:

1. **Input is well-defined**
   - Clear binary content
   - Documented structure (run1 + gap + run2)

2. **Canonical encoding verified**
   - Generated using z855.ts canonical encoder
   - Matches expected patterns
   - Uses documented escape types

3. **Both runs are visible as raw bytes**
   - First run appears as "AAAA..." or "BBBB..." etc
   - Second run appears as "CCCC..." or "DDDD..." etc
   - Raw bytes are human-readable in encoding

4. **Gap is encoded efficiently**
   - Uses canonical escape selection rules
   - Minimal output length for given input

5. **Round-trip verification**
   - `decode(encode(input)) === input` for z855.ts
   - `decode(encode(input)) === input` for min.mjs
   - Both decoders handle the encoding correctly

6. **Alignment edge cases**
   - Tests at block boundaries (positions 0, 4, 8, etc.)
   - Tests at non-aligned positions (positions 1, 2, 3, 5, etc.)

## File Naming Convention

```
test-cases/gap-<gapsize>-<run1len>-<run2len>[-pos<position>].input
test-cases/gap-<gapsize>-<run1len>-<run2len>[-pos<position>].encoded
```

Examples:
- `gap-0-4-4.input` - "AAAABBBB" (back-to-back 4-byte runs)
- `gap-1-5-5.input` - "AAAAA" + 1 zero + "BBBBB"
- `gap-4-8-8.input` - "AAAAAAAA" + 4 zeros + "BBBBBBBB"

## Expected Patterns

### Gap 0 (back-to-back)
- Combined length ≥ 8: Should use `|` escape for entire run
- Combined length = 7: Should use `~` escape for entire run
- Combined length = 6: Should use `_` escape for entire run
- Combined length = 5: Should use `;` escape for entire run
- Combined length = 4: Should use `,` escape for entire run

### Gap 1-3 (partial block)
- First run: Appropriate escape with raw bytes
- Gap: Encoded as trailing bytes of first escape or prefix of second
- Second run: Appropriate escape with raw bytes
- Total: Two separate escapes with encoded gap between

### Gap 4 (one block)
- First run: Escape with raw bytes
- Gap: Either pure Z85 "00000" or `,xxxx` escape (canonical rules apply)
- Second run: Escape with raw bytes

### Gap 5+ (multiple blocks)
- First run: Escape with raw bytes
- Gap: Multiple Z85 blocks or escapes
- Second run: Escape with raw bytes

## Review Checklist

Before accepting test results:

- [ ] All gap sizes covered (0, 1, 2, 3, 4, 5, 8, 16)
- [ ] All run lengths covered (4, 5, 6, 7, 8)
- [ ] Input files are correct binary
- [ ] Encodings match z855.ts canonical output
- [ ] Both runs visible in every encoding
- [ ] Round-trip works for all cases
- [ ] Decoders (TypeScript and min.mjs) handle all cases
- [ ] Edge cases documented (unusual choices, optimizations)
- [ ] Results added to test suite
