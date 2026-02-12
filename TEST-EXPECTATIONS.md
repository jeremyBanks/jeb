# Extended Z85 — Test Expectations

*Companion to DESIGN-CONSTRAINTS.md. Describes the test cases an implementation
must cover, without prescribing correct outputs or implementation details.*

## Guiding Principle

Every test should verify **round-trip correctness**: `decode(encode(data)) == data`.
This is the single most important property. If an implementation passes all
round-trip tests, most other bugs are caught transitively.

## 1. Standard Z85 Baseline

Extended Z85 is a superset of standard Z85. These tests verify the foundation.

- **Empty input** → empty output
- **Exactly 4 bytes** (one complete block)
- **Exactly 8 bytes** (two complete blocks)
- **Partial final block:** 1, 2, and 3 bytes (we lift the ZeroMQ 4-byte-multiple restriction)
- **5, 6, 7 bytes** (one complete block + partial)
- **All-zero input** (4 bytes, 8 bytes)
- **All-0xFF input** (4 bytes, 8 bytes)
- **Every byte value:** encode/decode a single block containing each byte 0x00-0xFF
  at each position within the block (position 0, 1, 2, 3)
- **Standard Z85 output contains no escape characters:** verify the output is
  pure Z85 alphabet when encoding binary data

## 2. Raw Section Basics

- **Block-aligned raw section:** 4 bytes of printable ASCII surrounded by
  binary data (e.g., `[binary4][ASCII4][binary4]` → Z85 + escape + raw + Z85)
- **8-byte block-aligned raw section**
- **12-byte block-aligned raw section**
- **Large block-aligned raw section** (e.g., 40+ bytes)
- **All-printable input:** entire input is printable ASCII, various lengths
  (4, 8, 12, 20, 100 bytes)
- **No-printable input:** entire input is binary, verify no raw sections appear
- **Mixed content:** alternating runs of binary and printable ASCII
- **Minimum raw section:** 4 bytes — verify the encoder uses raw passthrough
  even though there are zero character savings (transparency value)

## 3. Mid-Block Boundaries (R2 — Required)

These are the critical tests. An implementation that only supports block-aligned
raw sections is **incomplete** per §1 R2.

### Entry boundaries (raw section starts mid-block)

- **1-byte entry cut:** 3 bytes of binary, then printable ASCII begins.
  The encoder should emit 1 partial Z85 character for the binary byte, then
  begin raw passthrough.
- **2-byte entry cut:** 2 bytes of binary, then printable ASCII
- **3-byte entry cut:** 1 byte of binary, then printable ASCII
- **Entry at stream start with offset:** input begins with 1-3 non-printable
  bytes followed by a long printable run

### Exit boundaries (raw section ends mid-block)

- **1-byte exit cut:** printable ASCII followed by 3 bytes of binary
- **2-byte exit cut:** printable ASCII followed by 2 bytes of binary
- **3-byte exit cut:** printable ASCII followed by 1 byte of binary

### Both boundaries non-aligned

- **Entry AND exit mid-block:** e.g., `[bin2][ASCII7][bin1]` — the raw section
  doesn't start or end on a block boundary
- **Various combinations:** (1,1), (1,2), (1,3), (2,1), (2,2), (2,3), (3,1),
  (3,2), (3,3) entry/exit offset pairs, each with enough printable bytes in
  between to have positive budget

### Stability-dependent cases

- **Stable entry byte:** choose a byte value where the leading Z85 character is
  stable (68% of values — e.g., 0x00, which maps to a single leading digit).
  Verify the encoder takes the mid-block cut.
- **Unstable entry byte:** choose a byte value that straddles an 85^4 boundary
  (32% of values). Verify the encoder either disambiguates correctly or falls
  back to a different cut point.
- **Exit disambiguation from raw context:** verify the decoder correctly uses
  recently-decoded raw bytes to disambiguate exit boundary characters

## 4. Non-Aligned Lengths

Raw sections should not be constrained to multiples of 4 bytes.

- **5-byte raw section** (non-aligned, budget=2)
- **6-byte raw section**
- **7-byte raw section**
- **9-byte raw section** (budget=3, mid-block both ends feasible)
- **10, 11 byte raw sections**
- **13-byte raw section** (budget=4+, comfortable)
- **Odd lengths at various alignments:** e.g., 5 bytes starting at block
  offset 1, 7 bytes starting at block offset 3

## 5. Position Invariant (§3 P1)

- **Z85 blocks unchanged:** for input with mixed binary and ASCII regions,
  verify that every complete Z85 block that remains Z85-encoded produces the
  exact same characters as standard Z85 encoding of the full input
- **Output never longer:** extended encoding length ≤ standard Z85 length for
  the same input
- **Position alignment:** the Nth Z85 block in extended output occupies the same
  character positions as in standard Z85 (accounting for shorter output from
  raw sections)

## 6. Escape Characters & Length Encoding

- **Escape character detection:** verify non-Z85 characters in output are only
  escape characters (`_`, `~`, or whichever the implementation chose)
- **Length encoding round-trip:** for various raw section lengths, verify the
  length is correctly encoded and decoded
- **Maximum raw section length:** if the implementation has a max, verify
  sections at exactly the max length work, and longer runs are properly split
- **"Raw to end" escape** (if implemented): verify a raw section that extends
  to the end of the input works correctly, especially with non-aligned endings

## 7. Edge Cases

- **Single byte input** (just 1 byte, not printable)
- **Single printable byte** (just 1 printable byte — may or may not become raw,
  depends on budget)
- **Escape character as input byte:** `_` and `~` appear in the input data.
  Verify correct handling — these may or may not be eligible for raw
  passthrough depending on the implementation's policy
- **Z85 alphabet characters in raw sections:** printable bytes that happen to
  also be in the Z85 alphabet should work in raw sections
- **Raw section containing only escape characters:** e.g., `____` or `~~~~`
- **Consecutive raw sections:** the decoder should handle two raw sections
  back-to-back with no Z85 gap (even if the encoder wouldn't normally produce
  this — decoder permissiveness per R4)
- **Very long input** (e.g., 10KB+) with mixed content
- **Alternating single-byte binary / single-byte printable** — adversarial
  pattern where raw passthrough may not be beneficial

## 8. Raw Byte Eligibility (R3)

- **Printable ASCII in raw sections:** bytes 0x20-0x7E should generally be
  eligible (exact policy is encoder-dependent, see §0 "Raw byte values")
- **Non-printable bytes are Z85-encoded:** bytes outside the printable range
  should not appear in raw sections under the default policy
- **Escape characters in raw sections:** implementation-dependent — some may
  exclude escape chars from raw eligibility to simplify decoder logic,
  others may allow them since length-prefixed sections are unambiguous
- **Decoder accepts any raw bytes:** regardless of encoder policy, the decoder
  should handle raw sections containing any byte value (including non-printable)

## 9. Decoder Robustness

- **Standard Z85 input:** valid standard Z85 (no escape characters) decodes
  correctly — extended Z85 is a superset
- **Invalid escape sequence:** malformed escape prefix should produce a clear
  error, not silent corruption
- **Truncated raw section:** raw section length says N bytes but input ends
  early — should error
- **Invalid Z85 characters** (not in alphabet and not escape): should error

## 10. Property-Based Testing (Optional but Recommended)

If using a property-based testing framework (e.g., proptest, quickcheck):

- **Round-trip:** for arbitrary byte sequences, `decode(encode(data)) == data`
- **Output is printable ASCII:** all output bytes are in the printable ASCII
  range (Z85 alphabet + escape characters)
- **Output length bound:** `len(encode(data)) <= len(standard_z85_encode(data))`
- **Superset property:** encoding with raw sections disabled produces
  byte-identical output to standard Z85
- **Idempotent decode:** `decode(output)` produces the same result regardless
  of which valid encoding of the input was used (if multiple valid encodings
  exist)
