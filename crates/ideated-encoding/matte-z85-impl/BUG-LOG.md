# Bug Log - Z85 Implementation

## 2026-02-13 11:45 AM - Exit Block-Aligned Decode Failure

### Test: `test_exit_block_aligned`
**Input:** `b"     \x00\x00\x00\x00"` (5 spaces + 4 zeros)  
**Encoded:** `_\u{5}     000000` (13 bytes)  
**Expected:** Should decode successfully  
**Actual:** `Err(TruncatedInput)`

### Analysis
- Escape: `_` (block-aligned) ✓
- Length: 5 ✓
- Raw section: 5 spaces ✓
- After raw: `000000` (6 chars)

**Problem:** After 5-byte raw section, we have 4 bytes remaining. These should encode as:
- Standard Z85 block: 4 bytes → 5 chars
- BUT we're seeing 6 chars output

### Hypothesis
The mid-block exit implementation might be emitting an extra character, or the decoder is misinterpreting the structure.

### Next Steps
1. Check the exit implementation in src/lib.rs
2. Verify what `can_exit_opportunistic` returns for [0,0,0,0]
3. May need to fix implementation or adjust test expectation
4. Document whether this is encoder bug or decoder bug

---

## Investigation Needed
- [ ] Read exit implementation code
- [ ] Test `can_exit_opportunistic([0,0,0,0])`
- [ ] Determine if bug is in encoder or decoder
- [ ] Fix implementation or update test

---

## 2026-02-13 11:50 AM - Mid-Block Budget Violation

### Test: `test_midblock_budget_usage`
**Input:** `b"\x32     "` (1 byte + 5 spaces = 6 bytes total)  
**Standard Z85:** 8 chars  
**Encoded:** 9 bytes  
**Problem:** Exceeds standard Z85 length by 1 byte

### Analysis
Mid-block entry + raw passthrough:
- 2 chars (partial Z85 for byte 0x32)
- 1 char (escape `~`)
- 1 byte (length 5)
- 5 bytes (raw spaces)
- **Total: 2 + 1 + 1 + 5 = 9 bytes**

Standard Z85 for 6 bytes: `ceil(6 × 5/4) = 8 bytes`

**Budget violation: +1 byte (12.5% overhead)**

### Design Question
Is this acceptable? The encoding is transparent (shows spaces literally), but violates the "never longer than standard Z85" constraint.

Options:
1. Accept it (transparency > strict length bound)
2. Don't use mid-block entry when it would violate budget
3. Adjust budget calculations for mid-block cases

This needs Jeremy's input on design priorities.

---

## 2026-02-13 11:55 AM - Length Byte Semantics (256 vs 255 max)

### Observation
Encoder uses length byte 0-255, where:
- 1-255: literal length
- 0: means 256 bytes (wrap-around)

### Behavior
- 255 bytes → length=255 (1 section)
- 256 bytes → length=0 (1 section, decoded as 256)
- 300 bytes → ??? (needs testing)

### Design Question
Is this intentional? Two interpretations:
1. **Wrap-around**: length=0 means 256, allows 1-256 byte sections
2. **Split required**: length=0 invalid, must split at 255 bytes

Current tests show option 1 (wrap-around) is implemented.

### Impact
- If wrap-around intended: max single section = 256 bytes
- Large sections (300+) still need split logic
- Decoder must handle length=0 correctly

Needs clarification in design doc.
