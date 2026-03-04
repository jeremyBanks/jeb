#!/usr/bin/env python3
"""
Z85 Mid-Block Transition Stability Analysis

For each position where we might cut a Z85 block to start a raw section,
compute:
1. How often is the partial Z85 output "stable" (determined regardless of
   unknown future bytes)?
2. When stable, can the decoder unambiguously recover the original byte(s)?
3. How many disambiguation bits are needed when NOT stable?

Z85 encodes 4 bytes as a big-endian u32 into 5 base-85 digits.
The digits go from most-significant to least-significant (big-endian).
"""

Z85_ALPHABET = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#"

def z85_encode_block(b0, b1, b2, b3):
    """Encode 4 bytes to 5 Z85 characters."""
    value = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    chars = []
    for _ in range(5):
        chars.append(Z85_ALPHABET[value % 85])
        value //= 85
    return bytes(reversed(chars))

def z85_char(value, position):
    """Get the Z85 character at a specific position for a u32 value."""
    # Position 0 = most significant digit
    divisors = [85**4, 85**3, 85**2, 85, 1]
    return (value // divisors[position]) % 85


print("=" * 70)
print("Z85 MID-BLOCK STABILITY ANALYSIS")
print("=" * 70)

# ==========================================================================
# Analysis 1: Cut after 1 byte (positions 0 in output = char 0 determined?)
# ==========================================================================
# Known: byte 0 (b0). Unknown: bytes 1,2,3.
# V_min = b0 << 24 | 0x000000
# V_max = b0 << 24 | 0xFFFFFF
# Character 0 = V // 85^4
# Stable if floor(V_min / 85^4) == floor(V_max / 85^4)

print("\n--- Cut after 1 byte (1 char potentially stable) ---")
divisor = 85**4  # 52,200,625
uncertainty = 0xFFFFFF  # 16,777,215

stable_count = 0
stable_bytes = []
unstable_bytes = []
# For stable cases, check decoder consistency:
# decoder sees char c0 and must infer b0.
# Multiple b0 values can produce the same c0.
# Convention: decoder picks smallest b0 that produces c0.
decoder_consistent = 0

for b0 in range(256):
    v_min = b0 << 24
    v_max = v_min | uncertainty
    c_min = v_min // divisor
    c_max = v_max // divisor
    if c_min == c_max:
        stable_count += 1
        stable_bytes.append(b0)
        # Check decoder consistency: what b0 would decoder infer from c_min?
        # Decoder sees c0 = c_min, infers b0 = smallest b0 that gives c0
        # b0_inferred = ceil(c_min * divisor / 2^24) ... actually:
        # c0 = floor(V / 85^4) where V = b0*2^24 + unknown
        # For decoder: given c0, b0_min = ceil(c0 * 85^4 / 2^24)
        # But simpler: check if this b0 IS the smallest that produces c_min
        inferred_b0 = None
        for test_b0 in range(256):
            test_v_min = test_b0 << 24
            test_c = test_v_min // divisor
            if test_c == c_min:
                inferred_b0 = test_b0
                break
        if inferred_b0 == b0:
            decoder_consistent += 1
    else:
        unstable_bytes.append(b0)

print(f"Stable: {stable_count}/256 ({stable_count/256*100:.1f}%)")
print(f"Decoder-consistent (stable + canonical): {decoder_consistent}/256 ({decoder_consistent/256*100:.1f}%)")
print(f"Usable mid-block cuts after 1 byte: {decoder_consistent}/256")
print(f"\nUnstable byte values: {unstable_bytes[:20]}{'...' if len(unstable_bytes) > 20 else ''}")

# How many distinct c0 values map to multiple b0 values?
c0_to_b0s = {}
for b0 in range(256):
    v = b0 << 24
    c0 = v // divisor
    c0_to_b0s.setdefault(c0, []).append(b0)

multi_b0 = {c: bs for c, bs in c0_to_b0s.items() if len(bs) > 1}
print(f"\nZ85 char 0 values that map to multiple b0: {len(multi_b0)}")
for c, bs in sorted(multi_b0.items()):
    print(f"  c0={c} ({chr(Z85_ALPHABET[c])}): b0 = {bs}")

# ==========================================================================
# Analysis 2: Cut after 2 bytes
# ==========================================================================
print("\n--- Cut after 2 bytes (chars 0-1 potentially stable) ---")
# Known: b0, b1. Unknown: b2, b3.
# V_min = (b0<<24 | b1<<16)
# V_max = V_min | 0xFFFF
# Char 0 stable if floor(V_min/85^4) == floor(V_max/85^4)
# Char 1 stable if floor(V_min/85^3)%85 == floor(V_max/85^3)%85
# But char 1 stability requires char 0 stability.

divisor1 = 85**3  # 614,125
uncertainty2 = 0xFFFF  # 65,535

c0_stable = 0
c0c1_stable = 0
c0c1_consistent = 0

for b0 in range(256):
    for b1 in range(256):
        v_min = (b0 << 24) | (b1 << 16)
        v_max = v_min | uncertainty2
        
        c0_min = v_min // (85**4)
        c0_max = v_max // (85**4)
        
        if c0_min == c0_max:
            c0_stable += 1
            c1_min = (v_min // (85**3)) % 85
            c1_max = (v_max // (85**3)) % 85
            if c1_min == c1_max:
                c0c1_stable += 1
                # Check decoder consistency for 2-byte case
                # Decoder sees c0, c1 and must recover b0, b1
                # The value represented by c0*85^4 + c1*85^3 should let us
                # determine b0,b1 uniquely (pick canonical = smallest pair)
                encoded_prefix = c0_min * (85**4) + c1_min * (85**3)
                # b0,b1 from this: b0 = encoded_prefix >> 24, but there's a range
                # Actually the decoder just needs floor(encoded_prefix_value / 2^16)
                # to equal b0*256+b1
                inferred_pair = encoded_prefix >> 16
                actual_pair = (b0 << 8) | b1
                # But encoded_prefix is the BASE of the range, not exact
                # The canonical decode is: smallest (b0,b1) that produces these c0,c1
                # That's: b0*256+b1 = floor(c0*85^4 + c1*85^3 / 2^16)
                # Hmm, let me just check directly
                canonical_v = c0_min * (85**4) + c1_min * (85**3)
                canonical_pair = canonical_v >> 16
                if canonical_pair == actual_pair:
                    c0c1_consistent += 1

total_pairs = 256 * 256
print(f"Char 0 stable: {c0_stable}/{total_pairs} ({c0_stable/total_pairs*100:.1f}%)")
print(f"Chars 0+1 stable: {c0c1_stable}/{total_pairs} ({c0c1_stable/total_pairs*100:.1f}%)")
print(f"Chars 0+1 decoder-consistent: {c0c1_consistent}/{total_pairs} ({c0c1_consistent/total_pairs*100:.1f}%)")

# ==========================================================================
# Analysis 3: Cut after 3 bytes
# ==========================================================================
print("\n--- Cut after 3 bytes (chars 0-2 potentially stable) ---")
# Known: b0, b1, b2. Unknown: b3.
# V_min = (b0<<24 | b1<<16 | b2<<8)
# V_max = V_min | 0xFF
# Much higher stability since only 1 byte unknown

uncertainty3 = 0xFF
divisor2 = 85**2  # 7,225

# Sample rather than exhaustive (16M combinations)
import random
random.seed(42)
SAMPLES = 100000

c012_stable = 0
c012_consistent = 0
c0123_stable = 0  # all 4 chars stable (only possible if b3 doesn't matter)

for _ in range(SAMPLES):
    b0 = random.randint(0, 255)
    b1 = random.randint(0, 255)
    b2 = random.randint(0, 255)
    
    v_min = (b0 << 24) | (b1 << 16) | (b2 << 8)
    v_max = v_min | uncertainty3
    
    # Check chars 0,1,2 stability
    chars_min = []
    chars_max = []
    v = v_min
    for d in [85**4, 85**3, 85**2, 85, 1]:
        chars_min.append(v // d)
        v = v % d
    v = v_max
    for d in [85**4, 85**3, 85**2, 85, 1]:
        chars_max.append(v // d)
        v = v % d
    
    if chars_min[:3] == chars_max[:3]:
        c012_stable += 1
        # Check decoder consistency
        canonical_v = chars_min[0] * 85**4 + chars_min[1] * 85**3 + chars_min[2] * 85**2
        canonical_triple = canonical_v >> 8
        actual_triple = (b0 << 16) | (b1 << 8) | b2
        if canonical_triple == actual_triple:
            c012_consistent += 1
        
        if chars_min[3] == chars_max[3]:
            c0123_stable += 1

print(f"Chars 0-2 stable: ~{c012_stable/SAMPLES*100:.1f}% (sampled {SAMPLES})")
print(f"Chars 0-2 decoder-consistent: ~{c012_consistent/SAMPLES*100:.1f}%")
print(f"Chars 0-3 all stable: ~{c0123_stable/SAMPLES*100:.1f}%")

# ==========================================================================
# Analysis 4: Padding budget
# ==========================================================================
print("\n" + "=" * 70)
print("PADDING BUDGET ANALYSIS")
print("=" * 70)
print("""
For N raw bytes, standard Z85 produces ceil(N * 5 / 4) characters.
Raw passthrough uses N characters.
The difference is the "padding budget" available for:
- Escape character(s)
- Disambiguation bits for mid-block cuts
- Alignment padding
""")

print(f"{'N bytes':>8} | {'Z85 chars':>9} | {'Raw chars':>9} | {'Budget':>6} | {'Budget - 1 esc':>14}")
print("-" * 60)
for n in list(range(1, 21)) + [32, 50, 64, 100, 128, 256]:
    z85_chars = -(-n * 5 // 4)  # ceil division
    budget = z85_chars - n
    budget_minus_esc = budget - 1  # at least 1 char for escape
    print(f"{n:>8} | {z85_chars:>9} | {n:>9} | {budget:>6} | {budget_minus_esc:>14}")

# ==========================================================================
# Analysis 5: Disambiguation bits needed
# ==========================================================================
print("\n" + "=" * 70)
print("DISAMBIGUATION BITS FOR MID-BLOCK CUTS")
print("=" * 70)
print("""
When cutting after K bytes of a 4-byte block:
- K=1: 3 unknown bytes = 24 bits of ambiguity
  But char 0 determines b0 to within ceil(2^32/85) = ~50.5M range
  Actual ambiguity: ~24 bits minus what char 0 pins down
  
- K=2: 2 unknown bytes = 16 bits
- K=3: 1 unknown byte = 8 bits

When a cut is NOT stable, we need to encode the "lost" information.
The lost info is: which of the possible byte values (that all map to
the same partial Z85 output) is the correct one.
""")

# For 1-byte cut: how many bits does char0 pin down?
# char0 determines b0 to within a range of at most ceil(256/5) = 52 values
# (since 85^4 divides 2^32 unevenly)
# Actually: 2^32 / 85 = 50,529,027.something, so each char0 value covers
# about 50.5M out of 4.3B, which is about 4.89 values of b0.
# More precisely: for each c0, how many distinct b0 values produce it?

print("Char0 → b0 mapping (1-byte cut):")
for c, bs in sorted(c0_to_b0s.items()):
    if len(bs) <= 6:
        print(f"  c0={c:2d} ({chr(Z85_ALPHABET[c])}): {len(bs)} b0 values: {bs}")
    else:
        print(f"  c0={c:2d} ({chr(Z85_ALPHABET[c])}): {len(bs)} b0 values: {bs[:3]}...{bs[-3:]}")

print(f"\nDistribution of b0-per-c0:")
from collections import Counter
size_counts = Counter(len(bs) for bs in c0_to_b0s.values())
for size, count in sorted(size_counts.items()):
    print(f"  {count} char0 values map to {size} b0 values")
    
# Bits needed for disambiguation
import math
for size, count in sorted(size_counts.items()):
    bits = math.ceil(math.log2(size)) if size > 1 else 0
    print(f"  → {bits} bits needed to disambiguate among {size} candidates")

# For 2-byte cut: similar but for (b0,b1) pairs
print("\nFor 2-byte cut:")
print(f"  85^2 = 7225 possible (c0,c1) pairs encode 65536 (b0,b1) pairs")
print(f"  Average pair count: {65536/7225:.1f}")
print(f"  Max pair count: {math.ceil(65536/7225)}")
print(f"  Bits to disambiguate: {math.ceil(math.log2(math.ceil(65536/7225)))}")

print("\nFor 3-byte cut:")
print(f"  85^3 = 614125 possible (c0,c1,c2) triples encode 16777216 (b0,b1,b2) triples")
print(f"  Average triple count: {16777216/614125:.1f}")
print(f"  Max triple count: {math.ceil(16777216/614125)}")
print(f"  Bits to disambiguate: {math.ceil(math.log2(math.ceil(16777216/614125)))}")
