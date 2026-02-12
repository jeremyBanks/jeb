#!/usr/bin/env python3
"""
Z85 Endianness Analysis v2 — Corrected

The Z85 block encoding is FIXED: [b0, b1, b2, b3] → u32 = b0*2^24 + b1*2^16 + b2*2^8 + b3
This doesn't change.

The endianness question is: when we cut a block after K bytes, which
Z85 characters do we emit?

Option A (BE/leading): Emit the first K+1 characters (c0..cK).
  These are determined mainly by the high-order bytes (b0..bK-1).
  The known bytes ARE the high-order bytes.
  
Option B (LE/trailing): Emit the last K+1 characters (c(4-K)..c4).
  These are determined mainly by the low-order bytes.
  But wait — the known bytes (b0..bK-1) are still the FIRST bytes
  in memory, which are the HIGH-order bytes in the Z85 u32.
  
So "LE" doesn't mean we swap byte order in the Z85 computation.
It means: for the bytes we're cutting off (the ones that become raw),
we choose to encode the REMAINDER differently.

Actually, let me reconsider the whole framing.

The real question:
- We have bytes [b0, b1, b2, b3]
- We want to show some as raw and encode the rest as partial Z85
- The partial Z85 chars need to be stable (determined by the encoded bytes alone)

Case: "Cut after b0, show b1,b2,b3 as raw"
  Standard Z85 would encode all 4 as V = b0*2^24 + b1*2^16 + b2*2^8 + b3
  We want to encode just b0 as partial Z85.
  
  BE approach: Treat b0 as high-order. V_partial = b0 * 2^24.
    Emit char 0 = V_partial // 85^4.
    Stable if floor((b0*2^24)/85^4) == floor((b0*2^24 + 0xFFFFFF)/85^4)
    → 68% of b0 values.
    
  LE approach: Treat b0 as low-order. V_partial = b0.
    Emit char 4 = V_partial % 85. 
    Stable if (b0) % 85 == (b0 + 0xFFFFFF00) % 85
    Since 0xFFFFFF00 % 85 = 0, this is ALWAYS stable.
    → 100% stable!
    
  BUT: in the LE approach, the emitted char (c4) would NOT match what
  standard Z85 would produce for the full block! Because in standard Z85,
  c4 = (b0*2^24 + b1*2^16 + b2*2^8 + b3) % 85, which depends on ALL bytes.
  
  We're redefining what the partial Z85 means: instead of "the chars that
  standard Z85 would produce," it's "chars from an alternative computation
  that puts our known byte in a stable position."

So the REAL question is: does the partial Z85 output need to be a prefix
of what standard Z85 would produce? Or can it be something else entirely?

If it MUST be a prefix of standard Z85 output → BE is the only option
(because standard Z85 computes high-to-low, and the leading chars are
determined by the high bytes).

If it can be ANYTHING that's unambiguous and in Z85 alphabet → LE
(trailing char) is viable and gives 100% stability.

Let's compute both cases properly and also check: what about the position
invariant? The partial Z85 chars need to appear at specific positions in
the output stream.
"""

Z85_ALPHABET = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#"

print("=" * 70)
print("Z85 ENDIANNESS ANALYSIS v2 — CORRECTED")
print("=" * 70)

print("""
SETUP: We have 4 bytes [b0, b1, b2, b3] forming one Z85 block.
We want to cut after byte b0 and show b1, b2, b3 as raw.
We need to emit some Z85 character(s) for b0.

The key constraint: these chars must be STABLE (same regardless of b1,b2,b3)
AND the decoder must be able to recover b0 from them.
""")

# ================================================================
# Approach 1: BE — emit leading char(s), b0 in MSB
# ================================================================
print("--- Approach 1: BE (emit leading chars) ---")
print("Compute V = b0*2^24 + unknown. Emit c0 = V // 85^4.")
print()

stable_be = 0
decodable_be = 0  # stable AND canonical (decoder can recover b0)

c0_to_b0s = {}
for b0 in range(256):
    v_min = b0 << 24
    v_max = v_min | 0xFFFFFF
    c0_min = v_min // (85**4)
    c0_max = v_max // (85**4)
    
    c0_to_b0s.setdefault(c0_min, []).append(b0)
    
    if c0_min == c0_max:
        stable_be += 1
        # Canonical: decoder sees c0, assumes unknown bytes = 0
        # Gets b0_decoded = (c0 * 85^4) >> 24
        b0_decoded = (c0_min * (85**4)) >> 24
        if b0_decoded == b0:
            decodable_be += 1

print(f"Stable: {stable_be}/256 ({stable_be/256*100:.1f}%)")
print(f"Stable + canonical decode: {decodable_be}/256 ({decodable_be/256*100:.1f}%)")

# ================================================================
# Approach 2: LE — emit trailing char, b0 encoded via V % 85
# ================================================================
print("\n--- Approach 2: LE (emit trailing char) ---")
print("Compute V = unknown_high + b0. Emit c4 = V % 85.")
print("Since 0xFFFFFF00 mod 85 = 0, c4 = b0 % 85 always.")
print()

stable_le = 0
decodable_le = 0

c4_to_b0s = {}
for b0 in range(256):
    c4 = b0 % 85
    c4_to_b0s.setdefault(c4, []).append(b0)
    stable_le += 1  # always stable
    
    # Canonical decode: c4 → b0. But multiple b0 map to same c4!
    # b0 % 85 = c4 → b0 could be c4, c4+85, c4+170 (if < 256)
    # Convention: pick smallest → b0 = c4
    if b0 == c4:
        decodable_le += 1

print(f"Stable: {stable_le}/256 ({stable_le/256*100:.1f}%)")
print(f"Stable + canonical decode (b0 = c4): {decodable_le}/256 ({decodable_le/256*100:.1f}%)")
print(f"\nAmbiguity: each c4 maps to {256/85:.1f} b0 values on average")
print(f"  c4=0 → b0 in {c4_to_b0s[0]}")
print(f"  c4=1 → b0 in {c4_to_b0s[1]}")
print(f"  c4=84 → b0 in {c4_to_b0s[84]}")

# How many bits to disambiguate?
import math
max_ambiguity = max(len(vs) for vs in c4_to_b0s.values())
print(f"\nMax b0 candidates per c4: {max_ambiguity}")
print(f"Bits to disambiguate: {math.ceil(math.log2(max_ambiguity))}")

# ================================================================
# Approach 3: What if we DON'T require matching standard Z85?
# ================================================================
print("\n" + "=" * 70)
print("APPROACH 3: Independent partial encoding")
print("=" * 70)
print("""
What if the partial Z85 characters DON'T need to match what standard
Z85 would produce? They just need to:
1. Be in the Z85 alphabet (so the decoder knows they're Z85, not escape)
2. Encode the byte value unambiguously
3. Consume the right number of character positions

For 1 byte → we need 2 Z85 chars (85^2 = 7225 > 256).
This is the standard partial-block encoding.

The question is WHICH 2 chars and WHERE they go in the 5-char block.
""")

# Standard Z85 partial block for 1 byte:
# Pad [b0, 0, 0, 0], encode as V = b0 * 2^24, take first 2 chars
print("Standard partial (BE, leading 2 chars):")
for b0 in [0, 1, 32, 64, 100, 127, 128, 200, 255]:
    v = b0 << 24
    c0 = v // (85**4)
    c1 = (v // (85**3)) % 85
    print(f"  b0={b0:3d}: chars = {chr(Z85_ALPHABET[c0])}{chr(Z85_ALPHABET[c1])}")

# Alternative: encode b0 directly as 2 Z85 chars (base-85)
print("\nDirect base-85 encoding (2 chars, always unambiguous):")
for b0 in [0, 1, 32, 64, 100, 127, 128, 200, 255]:
    c0 = b0 // 85
    c1 = b0 % 85
    print(f"  b0={b0:3d}: chars = {chr(Z85_ALPHABET[c0])}{chr(Z85_ALPHABET[c1])}")

# ================================================================
# Approach 4: Entry vs Exit boundary analysis
# ================================================================
print("\n" + "=" * 70)
print("ENTRY vs EXIT BOUNDARY ANALYSIS")
print("=" * 70)
print("""
Entry boundary: [encoded bytes] [ESCAPE] [raw bytes...]
  The encoded bytes are a PREFIX of the Z85 block.
  In standard Z85 (BE), these are the leading chars.
  
Exit boundary: [...raw bytes] [Z85 suffix] [padding]
  The encoded bytes are a SUFFIX of the Z85 block.
  These are the trailing chars.

Key insight: at the ENTRY, the encoded bytes come BEFORE the raw section.
At the EXIT, they come AFTER.

For BE (leading chars), the entry is natural (known bytes → leading chars).
For the exit, the known bytes are at the END of the block → they determine
the TRAILING chars, not leading.

So entry and exit boundaries naturally use different "endianness":
  Entry → BE (prefix bytes → leading chars) — 68% stable
  Exit → effectively LE (suffix bytes → trailing chars)

Let's check exit stability explicitly.
""")

# Exit: bytes [b0, b1, b2, b3], b0,b1,b2 were raw, b3 needs Z85 encoding
# V = b0*2^24 + b1*2^16 + b2*2^8 + b3
# We know b3, unknown b0,b1,b2
# Last char c4 = V % 85
# Is c4 stable regardless of b0,b1,b2?

print("Exit after 3 raw bytes (b3 known, b0-b2 unknown):")
exit_stable_c4 = 0
for b3 in range(256):
    # V_min = b3, V_max = 0xFFFFFF00 + b3
    c4_min = b3 % 85
    c4_max = (0xFFFFFF00 + b3) % 85
    if c4_min == c4_max:
        exit_stable_c4 += 1

print(f"  Last char stable: {exit_stable_c4}/256 ({exit_stable_c4/256*100:.1f}%)")

print("\nExit after 2 raw bytes (b2,b3 known, b0,b1 unknown):")
import random
random.seed(42)
SAMPLES = 50000
exit2_c34_stable = 0
for _ in range(SAMPLES):
    b2 = random.randint(0, 255)
    b3 = random.randint(0, 255)
    v_min = (b2 << 8) | b3
    v_max = 0xFFFF0000 | (b2 << 8) | b3
    c3_min = (v_min // 85) % 85
    c3_max = (v_max // 85) % 85
    c4_min = v_min % 85
    c4_max = v_max % 85
    if c3_min == c3_max and c4_min == c4_max:
        exit2_c34_stable += 1

print(f"  Last 2 chars stable: ~{exit2_c34_stable/SAMPLES*100:.1f}%")

print("\nExit after 1 raw byte (b1,b2,b3 known, b0 unknown):")
exit1_c234_stable = 0
for _ in range(SAMPLES):
    b1 = random.randint(0, 255)
    b2 = random.randint(0, 255)
    b3 = random.randint(0, 255)
    v_min = (b1 << 16) | (b2 << 8) | b3
    v_max = 0xFF000000 | (b1 << 16) | (b2 << 8) | b3
    
    stable = True
    for pos in [2, 3, 4]:
        divs = [85**4, 85**3, 85**2, 85, 1]
        c_min = (v_min // divs[pos]) % 85
        c_max = (v_max // divs[pos]) % 85
        if c_min != c_max:
            stable = False
            break
    if stable:
        exit1_c234_stable += 1

print(f"  Last 3 chars stable: ~{exit1_c234_stable/SAMPLES*100:.1f}%")

# ================================================================
# Summary
# ================================================================
print("\n" + "=" * 70)
print("SUMMARY: Entry vs Exit stability")
print("=" * 70)
print("""
Entry boundary (prefix bytes → leading chars, BE):
  1 byte known: 68% stable (char 0)
  2 bytes known: 89% stable (chars 0-1) 
  3 bytes known: 96% stable (chars 0-2)

Exit boundary (suffix bytes → trailing chars):
  1 byte known (b3): 100% stable (char 4)
  2 bytes known (b2,b3): check above
  3 bytes known (b1,b2,b3): check above
  
This asymmetry is expected: the trailing Z85 digit depends on V mod 85,
which is dominated by the low-order bytes. The leading digit depends on
V / 85^4, dominated by high-order bytes.

Design implication: entry and exit boundaries have different natural
"best" conventions. Entry prefers BE (leading chars). Exit prefers
trailing chars (which is effectively LE from the boundary's perspective).
""")
