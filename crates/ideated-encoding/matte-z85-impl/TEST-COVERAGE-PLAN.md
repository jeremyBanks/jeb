# Z85 Test Coverage Plan
**Date:** 2026-02-13  
**Current Grade:** C- (from Sonnet review)  
**Goal:** Achieve comprehensive coverage of all design requirements

## Coverage Status

### ✅ Strong (Grade A)
- [x] Mid-block entry (1,2,3-byte boundaries)
- [x] Stability checks (b < 174)
- [x] Position invariant

### ⚠️ Partial (Grade B-C)
- [ ] Non-aligned lengths (only 5,6,8 tested; missing 7,9-11)
- [ ] Budget constraints (5-byte tested; missing 4-byte break-even, 9-byte boundary)
- [ ] Transparency/readability

### ❌ Weak/Missing (Grade D-F)
- [x] **Raw eligibility** - ✅ DONE: 5 comprehensive tests (11:42 AM)
- [ ] **Mid-block exit structure** - Only roundtrip, no structure verification
- [ ] **Consecutive raw sections** - R4 requirement, not tested
- [ ] **Edge cases** - Stream boundaries, 256+ byte splits
- [ ] **Exit opportunistic validation** - Does it skip when zero-padding fails?

## Test Gaps to Fill

### Priority 1: Critical Requirements
1. **Raw eligibility comprehensive test**
   - All printable ASCII (32-126) except Z85 alphabet and escapes
   - Verify non-printable (0-31, 127-255) always Z85-encoded
   - Test: `test_comprehensive_raw_eligibility`

2. **Mid-block exit structure verification**
   - Verify K bytes → K+1 trailing chars after raw section
   - Check opportunistic condition (only exit when zero-padding works)
   - Test: `test_exit_structure_verified`, `test_exit_opportunistic_failure`

3. **Consecutive raw sections (R4)**
   - Pattern: raw1 + raw2 with no Z85 between
   - Decoder must accept (though encoder should never produce)
   - Test: `test_consecutive_raw_sections`

### Priority 2: Budget Boundaries (§8)
4. **4-byte break-even**
   - Exactly 4 bytes raw, budget=1
   - Should NOT use raw (no net savings, we require 5+)
   - Test: `test_4_byte_no_raw_passthrough`

5. **9-12 byte boundary (budget=3)**
   - Test comfortable budget zone
   - Test: `test_9_to_12_byte_budget`

### Priority 3: Edge Cases
6. **Non-aligned length coverage**
   - Systematically test 1-16 bytes
   - Focus on 7,9-11 (currently missing)
   - Test: `test_all_lengths_1_to_16`

7. **Stream boundaries**
   - Raw at start of stream
   - Raw at end of stream
   - Test: `test_raw_at_stream_boundaries`

8. **Large raw sections**
   - 300+ bytes (requires split into multiple sections, max 255 each)
   - Test: `test_large_raw_section_split`

## Implementation Issues to Check

### Verify Against Design Doc
- [ ] R1: Non-aligned raw section lengths ✓ (but test coverage weak)
- [ ] R2: Mid-block boundary support ✓ (entry strong, exit weak)
- [ ] R3: Raw byte eligibility (default policy) - **needs verification**
- [ ] R4: Consecutive raw sections - **not tested**
- [ ] P1: Position invariant ✓
- [ ] §8: Budget (5+ bytes) ✓ (but 4-byte boundary not tested)

### Known Implementation Decisions
- Minimum raw section: 5 bytes (changed from 4)
- Escape characters: `_` (aligned), `~` (1-byte), `|` (2-byte), `,` (3-byte), `;` (reserved)
- Raw eligibility: printable ASCII NOT in Z85 alphabet, NOT escape chars

## Testing Strategy

1. **Add tests incrementally** - one gap at a time
2. **Run full suite after each addition** - ensure no regressions
3. **Document failures** - if tests reveal bugs, note them
4. **If bugs found** - may need to spawn sub-agents to fix implementation
5. **Keep this file updated** - track what's done, what works, what doesn't

## Session Log

### 2026-02-13 11:40 AM - Starting systematic coverage
- Current: 37/37 tests passing
- Sonnet review: C- grade
- Plan: Fill gaps starting with Priority 1

---

## Notes
- Keep commits small and focused
- Document reasoning for each test
- If implementation needs changes, coordinate with Jeremy first
