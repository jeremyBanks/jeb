#!/usr/bin/env python3
"""
Z85 — Bit-level analysis of partial block encoding.

Question: when encoding K bytes of a 4-byte block, are we leaving
efficiency on the table by thinking in terms of "leading chars" vs
"trailing chars"? Is there a bit-level trick we're missing?

Standard Z85:
  4 bytes (32 bits) → 5 chars of base-85 (~32.28 bits capacity)
  
For K known bytes:
  K=1: 8 bits of info → need 1+ chars (~6.4 bits/char in base-85)
  K=2: 16 bits → need 2+ chars (~12.8 bits)
  K=3: 24 bits → need 3+ chars (~19.2 bits)

The "leading chars" approach extracts chars from the MSB end of the
32-bit value. The "trailing chars" approach extracts from the LSB end.

But there's another possibility: what if we used a DIFFERENT base for
the partial encoding? Not base-85 at all, but something that exactly
matches the information content?

For K=1 (8 bits = 256 values):
  1 char in base-85 = 85 values (not enough!)
  2 chars in base-85 = 7225 values (way more than 256)
  2 chars in base-16 = 256 values (exact — this is just hex!)
  
  So with "leading chars": 1 char encodes 85 of 256 values.
  The other 171 values need the unknown bytes to disambiguate.
  That's why stability is only 68/82 ≈ 83%... wait, I calculated 68% before.
  
  Let me recheck.
"""

# Recheck: for K=1, how many byte values produce a stable leading char?
count_stable = 0
for b0 in range(256):
    # Check if char0 is the same for all possible b1, b2, b3
    chars = set()
    # Sample a few values (0, 127, 255 for each unknown byte)
    for b1 in [0, 127, 255]:
        for b2 in [0, 127, 255]:
            for b3 in [0, 127, 255]:
                v = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
                c0 = v // (85**4)
                chars.add(c0)
    if len(chars) == 1:
        count_stable += 1

print(f"K=1 stability (sampled): {count_stable}/256 = {count_stable/256*100:.1f}%")

# More precise: char0 = V // 85^4
# V ranges from b0*2^24 to b0*2^24 + 2^24 - 1
# char0 changes when V crosses a multiple of 85^4 = 52200625
# 2^24 = 16777216
# So the range [b0*2^24, (b0+1)*2^24 - 1] spans 16777216 values
# Number of 85^4 boundaries crossed = floor of range
import math
p = 85**4  # 52200625
count_exact = 0
for b0 in range(256):
    lo = b0 * (2**24)
    hi = (b0 + 1) * (2**24) - 1
    c_lo = lo // p
    c_hi = hi // p
    if c_lo == c_hi:
        count_exact += 1

print(f"K=1 stability (exact): {count_exact}/256 = {count_exact/256*100:.1f}%")

# Now the KEY question: what if instead of emitting char0 (which only
# captures ~6.4 bits and has 32% instability), we emitted the byte ITSELF
# using a different encoding within the Z85 alphabet?
#
# For K=1, we have 1 char position to work with (char0's slot).
# But we could also use 2 char positions (chars 0-1).
# With 2 Z85 chars we can encode 85^2 = 7225 values — way more than 256.
# So we could encode the byte value EXACTLY in 2 Z85 chars with no
# disambiguation needed at all!
#
# Cost: 2 chars instead of 1
# But wait — the position invariant says Z85 chars must be at their
# standard positions. If we use 2 chars for 1 byte, we're "spending"
# an extra position. Is that within budget?

print("\n" + "="*60)
print("ALTERNATIVE: Exact byte encoding in Z85 chars")
print("="*60)

# For the "leading char" approach with K=1:
# Standard: 1 char (char0), 68% stable, needs ~2 bits disambig when not
# Alternative: 2 chars (chars 0-1), 100% stable, no disambig needed
# But uses an extra char slot → reduces budget by 1

# For K=1 entry with budget comparison:
for raw_n in [4, 8, 12, 20]:
    z85_chars = -(-raw_n * 5 // 4)
    budget = z85_chars - raw_n
    
    # "1 partial char" approach: 1 char for entry + 1 escape + raw + padding
    budget_1char = budget  # escape comes from budget
    # "2 partial chars" approach: 2 chars for entry + 1 escape + raw + padding  
    # BUT: the 2 chars come from the entry block's own 5-char allocation,
    # not from the raw section's budget
    # Actually, let me reconsider...
    
    print(f"\nRaw={raw_n}: Z85 chars={z85_chars}, raw budget={budget}")
    print(f"  1-char entry (68% stable): budget for escape+disambig = {budget}")
    print(f"  2-char entry (100% stable): costs 1 more char from entry block")

print("\n" + "="*60)
print("DEEPER: What about non-uniform bit allocation?")
print("="*60)

# The real question might be: Z85's base-85 digits don't align with
# byte boundaries. That's the fundamental source of the instability.
# 
# What if the partial encoding used a DIFFERENT base for the boundary
# chars? Like:
#   - Entry char: base-256 (just the raw byte, but encoded in Z85 alphabet)
#     But Z85 only has 85 chars, can't encode 256 values in 1 char.
#   - Entry chars: base-85 pair encoding the byte value directly
#     85^2 = 7225 >> 256, so 2 chars always works, 100% stable.
#
# This is different from "leading Z85 chars" because leading Z85 chars
# encode the byte's contribution to the 32-bit value, not the byte itself.
#
# The difference:
#   Leading char approach: c0 = (b0*2^24 + ???) // 85^4
#   Direct encoding: c0,c1 = b0 // 85, b0 % 85  (or any bijection)
#
# The direct encoding breaks the position invariant for that char position
# (it won't match what standard Z85 would produce). But we already said
# mid-block transition chars are exempt from position invariance!

print("""
KEY REALIZATION:

Since mid-block transition chars are ALREADY exempt from the position
invariant, we're FREE to use any encoding for them — not just the
"natural" leading/trailing Z85 chars.

Options for encoding K boundary bytes:
1. Natural leading/trailing chars (current approach)
   - Pro: simplest, reuses Z85 math
   - Con: 68-96% stable, needs disambiguation when not

2. Direct byte encoding in Z85 chars  
   - 1 byte → 2 Z85 chars (100% stable, costs 1 extra char)
   - 2 bytes → 3 Z85 chars (100% stable, costs 1 extra char)  
   - 3 bytes → 4 Z85 chars (100% stable, costs 1 extra char)
   - Pro: always exact, no disambiguation
   - Con: costs 1 extra char per boundary

3. Hybrid: use natural chars when stable, direct encoding otherwise
   - Pro: optimal on average
   - Con: decoder needs to know which convention was used (costs info)

Actually wait — option 2 math:
  1 byte (8 bits) needs ceil(8/6.4) = 2 chars (85^2=7225 >> 256) ✓
  2 bytes (16 bits) needs ceil(16/6.4) = 3 chars (85^3=614125 >> 65536) ✓
  3 bytes (24 bits) needs ceil(24/6.4) = 4 chars (85^4=52200625 >> 16777216) ✓

So direct encoding always costs K+1 chars for K bytes.
The natural approach costs K chars (but needs ~2 bits/byte disambig).

The tradeoff is:
  Natural: K chars + ~2K bits (from escape budget or disambiguation encoding)
  Direct: K+1 chars + 0 bits

For the tight 4-byte-aligned case (budget=1):
  Natural: can't do mid-block (no room for disambig)
  Direct: can't do mid-block (no room for extra char)
  → Neither works! Short sections must be block-aligned regardless.

For 8-byte (budget=2):
  Natural: 1 entry char + 1 escape + (disambig from escape choice) → works if 
           escape char encodes the disambig
  Direct: 2 entry chars + 1 escape = 3 chars of budget → doesn't fit (budget=2)!
  → Natural wins for tight budgets.

For 12-byte (budget=3):
  Natural: 1 escape + disambig → comfortable (budget has room)
  Direct: 2 entry chars would eat 1 extra → still fits
  → Both work, natural is more compact.
""")

print("CONCLUSION:")
print("The 'natural' leading/trailing char approach is more efficient")
print("for tight budgets because it uses fewer char slots (K vs K+1).")
print("The disambiguation cost (~2 bits/byte) is paid from the escape")
print("character budget, which is cheaper than a full extra Z85 char.")
print("")
print("The 'different level' optimization isn't in how we encode the")
print("boundary bytes — it's in how we spend the escape char's info bits.")
print("Each escape char choice gives us ~2.6-6.4 bits of info (depending")
print("on how many escape chars exist). Those bits can carry disambiguation")
print("more efficiently than adding extra Z85 chars.")
