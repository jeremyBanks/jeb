# Evaluation: haiku-6 Implementation

**Source:** `/Users/matte/z85-implementations/haiku-6/`
**Agent:** Claude Haiku 4.5, Round 2 of implementation race
**Test results:** 66 passing, 0 failing, 0 ignored
**Lines:** 668 total Rust (z85.rs: 377, extended.rs: 231, error.rs: 42, lib.rs: 14, main.rs: 4)
**Commits:** 9 (incremental development, bug fixes visible in history)

## Verdict: Viable reference implementation with caveats

haiku-6 is the only implementation across 8 agents (6 Round 1 + 4 Round 2) that
actually implements and passes mid-block boundary tests. It round-trips correctly
for all tested inputs including binary, printable ASCII, mixed content, partial
blocks, every byte value at every position, and 10KB stress tests.

## What It Gets Right

### 1. Round-trip correctness
66 tests, zero failures. Tests cover:
- Every byte value (0x00–0xFF) at each position within a 4-byte block
- Partial blocks (1, 2, 3 bytes)
- Mid-block entry at 1, 2, 3 byte offsets
- Mid-block exit at 1, 2, 3 byte offsets
- Combined mid-block entry + exit
- Non-aligned raw lengths (5, 6, 7, 9, 10, 11, 13 bytes)
- Escape chars in input data (`_`, `~`)
- All printable ASCII (0x20–0x7E)
- 10KB mixed data
- Consecutive raw sections
- Decoder error handling (truncated, invalid chars)

### 2. Mid-block cuts work
This is the §1 R2 hard requirement that every other agent deferred or ignored.
Example outputs showing it works:

```
Input: \x00\x01\x02 Hello    (3 binary + 5 printable)
Output: 009c( _004ello        (Z85 block + raw section)

Input: \x00\x01 HelloWorld    (2 binary + 10 printable)  
Output: 00bS4 _008lloWorld    (Z85 block + raw section)

Input: HelloW \xFF\xFE\xFD   (6 printable + 3 binary)
Output: _006HelloW %nI@       (raw section + Z85 block)
```

The encoder doesn't align raw sections to Z85 block boundaries — it starts raw
wherever the printable run begins and stops wherever it ends.

### 3. Architecture
Clean separation: `z85.rs` (standard), `extended.rs` (raw passthrough), `error.rs`.
Forward-only decoder. No global state. Reasonable error types.

## What It Gets Wrong (or Differently)

### 1. NOT mid-block transitions in the design doc sense
The doc's mid-block concept is about *sharing a Z85 block* between raw and encoded
data, with disambiguation bits to recover the split point. haiku-6 doesn't do this.

What haiku-6 actually does: raw sections are independent byte sequences. When a
raw section starts mid-block, the preceding bytes get encoded as a *complete Z85
block* (4 bytes → 5 chars) even though some of those bytes are printable. There's
no block-sharing or disambiguation.

This means for `\x00\x01 HelloWorld` (2 binary + "HelloWorld"):
- Encoder: Z85-encodes `\x00\x01He` as one block (5 chars), then raw section `lloWorld` (4+8=12)
- The `He` bytes get Z85-encoded even though they're printable
- Total: 17 chars. A true mid-block implementation could potentially do better.

This is still a valid approach — it satisfies the *spirit* of R2 (raw sections
don't need to be block-aligned) even if it doesn't implement the *mechanism*
described in §6-§8 of the design doc.

### 2. Fixed 3-char length encoding (wasteful)
Every raw section costs `1 (escape) + 3 (length) + N (data)` = N+4 chars.
Standard Z85 for N bytes costs `ceil(N/4) * 5` chars.

Break-even: N+4 ≤ ceil(N/4)*5
- N=4: 8 ≤ 5 → raw is WORSE (8 vs 5)
- N=5: 9 ≤ 10 → raw wins by 1
- N=8: 12 ≤ 10 → raw is WORSE (12 vs 10)
- N=12: 16 ≤ 15 → raw is WORSE (16 vs 15)
- N=16: 20 ≤ 20 → break even
- N=20: 24 ≤ 25 → raw wins by 1

The 3-char overhead means raw only wins for N ≥ 16 or at non-aligned lengths
where Z85 padding wastes space. The design doc envisions 1-2 length chars for
common cases, not 3. This is the biggest efficiency gap.

Example: `"Hello World Test"` (16 bytes) → 20 chars both ways. Zero savings.
Standard Z85 of same input is also 20 chars. The transparency benefit is the
only win here — the raw data is visible in the output.

### 3. Wide raw eligibility policy
`is_raw_eligible`: any byte 0x20–0x7E except `_`. This includes spaces, quotes,
commas, semicolons — characters NOT in the Z85 alphabet. The design doc's R3
leaves this as policy choice, but the wider policy means the encoded output
contains characters that standard Z85 would never produce. A decoder must
distinguish "is this Z85 or raw?" solely by the escape prefix.

This is actually fine for round-trip correctness (the escape + length framing
handles it), but it changes the output's compatibility profile. A standard Z85
decoder fed this output would misinterpret the raw sections.

### 4. Single escape character
Uses only `_`, not `_` + `~`. Forfeits the information bit from escape char
choice that the design doc identifies as a key optimization lever (§8).

### 5. Brute-force partial block decoding
`decode_partial_block` uses nested loops over all byte values (O(256^K) for K
unknown bytes). The 3-byte case is O(256³) ≈ 16M iterations per partial block.
This works but is glacial for production use. The design doc says "complexity is
an afterthought" so this is acceptable for a reference implementation, but it's
worth noting that mathematical inversion is O(1).

### 6. `encode_trailing_bytes` and `decode_trailing_chars` exist but are unused
The z85.rs module has helper functions for encoding/decoding trailing portions
of a Z85 block — suggesting the agent was thinking about proper mid-block
transitions — but `extended.rs` never calls them. The actual mid-block handling
just falls through to partial-block and full-block encoding.

## Comparison to Design Doc Requirements

| Requirement | Status | Notes |
|-------------|--------|-------|
| §1 R1: Non-aligned raw lengths | ✅ | 5,6,7,9,10,11,13-byte tests pass |
| §1 R2: Mid-block boundaries | ⚠️ | Works but via block completion, not block-sharing |
| §1 R3: Raw byte eligibility | ✅ | Policy is wider than Z85 alphabet (0x20-0x7E) |
| §1 R4: Consecutive raw sections | ✅ | Decoder handles; encoder doesn't produce |
| §3 P1: Position invariant | ⚠️ | Z85 blocks are correct, but block boundaries shift |
| §10: Length before data | ✅ | `_` + 3-char Z85 length + raw bytes |
| §11: Mid-block both boundaries | ⚠️ | Not in the disambiguation-bits sense |

## Bottom Line

**As a reference implementation for testing and iteration: yes, use it.** The
round-trip correctness is solid, the test coverage is the best of all 8 agents,
and it demonstrates that non-aligned raw sections can work.

**As the final format: no.** The 3-char length encoding kills efficiency for
short raw sections (the common case). The escape char info bits are unused. The
mid-block handling doesn't actually share Z85 blocks. These are all optimization
opportunities the design doc specifically addresses.

**Recommended use:** Merge as a reference/baseline in the repo. Use its test
suite as a compatibility target. Iterate on efficiency from this working base.
