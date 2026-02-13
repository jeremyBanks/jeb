# Bug Log - Z85 Implementation

## ✅ FIXED: 2026-02-13 11:45 AM - Exit Block-Aligned Decode Failure

### Test: `test_exit_block_aligned`
**Input:** `b"     \x00\x00\x00\x00"` (5 spaces + 4 zeros)  
**Encoded (before fix):** `_\u{5}     000000` (13 bytes, 6 chars after raw)  
**Encoded (after fix):** `_\u{5}     00000` (12 bytes, 5 chars after raw)  
**Error (before):** `Err(TruncatedInput)`  
**Status (after):** ✓ Decodes successfully

### Root Cause
Opportunistic exit was fragmenting 4-byte blocks:
- Checked "bytes to block boundary" (3)
- Encoded 3 bytes as partial (4 chars)
- Remaining 1 byte encoded as partial (2 chars)
- Total: 6 chars instead of 5 (full block)
- Decoder rejected: 5 chars OK, but 1 remaining char invalid (partials need 2-4 chars)

### Fix (Commit b1582e5c)
Skip opportunistic exit when remaining bytes fill complete blocks:
```rust
if remaining >= 4 && remaining % 4 == 0 {
    // Let main loop encode as full blocks
}
```

**Result:** Encoder emits valid output, decoder processes correctly

---

## ✅ FIXED: 2026-02-13 11:50 AM - Mid-Block Budget Violation

### Test: `test_midblock_budget_usage`
**Input:** `b"\x32     "` (6 bytes)  
**Standard Z85:** 8 chars  
**Encoded (before fix):** 9 chars (P1 VIOLATION)  
**Encoded (after fix):** 8 chars ✓

### Root Cause
Encoder used mid-block entry without checking if it would violate position invariant:
- 1 byte partial: 2 chars
- Escape + length: 2 chars
- Raw: 5 bytes
- Total: 9 chars > 8 chars standard (VIOLATION)

### Fix (Commit bbab3101)
Added `check_midblock_budget()` to verify P1 before using mid-block:
```rust
fn check_midblock_budget(leading_bytes: &[u8], raw_len: usize, total_len: usize) -> bool {
    let midblock_cost = (leading_bytes.len() + 1) + 1 + 1 + raw_len;
    let standard_cost = (total_len * 5 + 3) / 4;
    midblock_cost <= standard_cost
}
```

**Result:** P1 hard requirement now enforced

---

## Investigation Needed
- [x] Read exit implementation code
- [x] Test `can_exit_opportunistic([0,0,0,0])`
- [x] Determine if bug is in encoder or decoder
- [x] Fix implementation ✓

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
