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
