#!/usr/bin/env python3
"""
Z85 — Exit boundary stability analysis (analytical approach).

Instead of brute-forcing 256^4 combinations, we can reason mathematically.

Z85 encoding: V = b0*2^24 + b1*2^16 + b2*2^8 + b3
  char4 = V mod 85
  char3 = (V // 85) mod 85
  char2 = (V // 85^2) mod 85
  ...

For exit boundaries, known bytes are at the END: b3 (1-byte), b2+b3 (2-byte), etc.
Unknown bytes are at the START: b0 (3-byte exit), b0+b1 (2-byte), b0+b1+b2 (1-byte).

KEY INSIGHT: The unknown bytes contribute U = (unknown high bytes) * 2^(8*K)
where K is the number of known bytes. The known bytes contribute L (a fixed value).
V = U + L, where U varies and L is fixed.

For char4 = V mod 85 = (U + L) mod 85.
This is stable iff U mod 85 is always the same, i.e., all possible U values
give the same U mod 85.

For 1-byte exit: U ranges over [0, 2^24 - 1] in steps of 2^8 (wait, no).
Actually: U = b0*2^24 + b1*2^16 + b2*2^8, L = b3.
U ranges over all multiples of... no, U = b0*2^24 + b1*2^16 + b2*2^8.
U takes every value from 0 to 2^32-256 in steps of 256? No.
U = (b0*65536 + b1*256 + b2) * 256.
So U is any multiple of 256 from 0 to (256^3-1)*256 = 16777215*256.
Actually U can be ANY value from 0 to 0xFFFFFF00 (in steps of 0x100? No!)
U = b0*0x01000000 + b1*0x00010000 + b2*0x00000100
The possible values of U are: all values of the form 0xBB_BB_BB_00
where each B is 0-255. That's all multiples of 256 in [0, 0xFFFFFF00].
That's 256^3 = 16777216 distinct values.

For char4 = (U + L) mod 85 to be stable, we need (U mod 85) to be the same
for ALL U that are multiples of 256 in [0, 0xFFFFFF00].

256 mod 85 = 256 - 3*85 = 256 - 255 = 1.

So U mod 85 = (U/256 * 256) mod 85 = (U/256 * 1) mod 85 = (U/256) mod 85.
Since U/256 ranges over all values 0 to 16777215, U/256 mod 85 takes ALL
values 0-84. Therefore U mod 85 takes all values 0-84.

Wait, that means (U + L) mod 85 takes all values 0-84 regardless of L!
char4 is NEVER stable for 1-byte exit?!

But we proved earlier that char4 IS 100% stable... Let me recheck.

Actually wait. 256 mod 85 = 1. So U mod 85 = (sum of byte * positional_weight mod 85).
U = b0*2^24 + b1*2^16 + b2*2^8
U mod 85:
  2^8 mod 85 = 256 mod 85 = 1
  2^16 mod 85 = 65536 mod 85: 85*770 = 65450, 65536-65450 = 86, 86 mod 85 = 1
  2^24 mod 85 = (2^16 * 2^8) mod 85 = (1 * 1) mod 85 = 1

So U mod 85 = (b0 + b1 + b2) mod 85.
This takes values 0 through 84 (since b0+b1+b2 ranges 0 to 765, covering
all residues mod 85 many times over).

So (U + b3) mod 85 cycles through ALL 85 values. char4 is NEVER stable!

But this contradicts our earlier finding of 100% stability...
"""

# Let me just verify empirically for b3=0
print("Empirical check: b3=0, all b0/b1/b2")
c4_values = set()
for b0 in range(256):
    for b1 in range(256):
        for b2 in range(256):
            v = (b0 << 24) | (b1 << 16) | (b2 << 8) | 0
            c4_values.add(v % 85)
print(f"  b3=0: char4 takes {len(c4_values)} distinct values: {sorted(c4_values)[:10]}...")

# OK so the analytical result says it should take all 85 values.
# But the PREVIOUS script said it was 100% stable.
# Let me check what the previous script actually tested...

# The previous "endianness" script checked LEADING chars under LE placement.
# The "endianness-v2" script... let me re-derive.
# 
# Actually, I think the 100% claim came from: 0xFFFFFF00 mod 85 = 0.
# This means: the CONTRIBUTION of the unknown bytes to V mod 85 is zero
# when the unknown bytes are all 0xFF. But that's just one specific case!
#
# The claim was: "V mod 85 depends only on the low byte regardless of 
# the upper bytes' values." Let's test THAT claim directly.

print("\nDirect test: does V mod 85 depend only on b3?")
print("For b3=42, checking all (b0,b1,b2)...")
c4_for_b3_42 = set()
for b0 in range(256):
    for b1 in range(256):
        for b2 in range(256):
            v = (b0 << 24) | (b1 << 16) | (b2 << 8) | 42
            c4_for_b3_42.add(v % 85)
print(f"  b3=42: char4 takes {len(c4_for_b3_42)} distinct values")

# Now the MATHEMATICAL analysis:
# V = b0*2^24 + b1*2^16 + b2*2^8 + b3
# V mod 85 = (b0*2^24 + b1*2^16 + b2*2^8 + b3) mod 85
# 
# 2^8 mod 85 = ?
print("\nKey modular arithmetic:")
for exp in [8, 16, 24]:
    val = (2**exp) % 85
    print(f"  2^{exp} mod 85 = {val}")

# If 2^8 mod 85 = 1, then:
# V mod 85 = (b0*1 + b1*1 + b2*1 + b3) mod 85 = (b0+b1+b2+b3) mod 85
# This is NOT determined by b3 alone!

print("\nSo V mod 85 = (b0 + b1 + b2 + b3) mod 85")
print("This is NOT determined by b3 alone — it depends on ALL bytes.")
print()
print("THE 100% STABILITY CLAIM WAS WRONG.")
print()
print("The error was: '0xFFFFFF00 mod 85 = 0' is true, but this only means")
print("that when b3=0 and all other bytes are 0xFF, the result is 0.")
print("It does NOT mean that V mod 85 is independent of the upper bytes.")
print()

# What IS true: 0xFFFFFF00 mod 85
val = 0xFFFFFF00 % 85
print(f"0xFFFFFF00 mod 85 = {val}")
# And: 0x00000100 mod 85
val2 = 0x00000100 % 85
print(f"0x00000100 mod 85 = {val2}")
val3 = 0x00010000 % 85
print(f"0x00010000 mod 85 = {val3}")
val4 = 0x01000000 % 85
print(f"0x01000000 mod 85 = {val4}")

print()
print("Since 2^8 ≡ 2^16 ≡ 2^24 ≡ 1 (mod 85),")
print("V mod 85 = (b0 + b1 + b2 + b3) mod 85")
print()
print("This means the trailing Z85 character depends on the SUM of all bytes mod 85.")
print("It is NOT stable — changing any byte changes the sum.")
print()
print("CORRECTED EXIT STABILITY: The trailing char has the SAME instability")
print("as the leading char, just expressed differently.")
print()

# So what IS the exit stability?
# For 1-byte exit (b3 known, b0/b1/b2 unknown):
# char4 = (b0+b1+b2+b3) mod 85
# Unknown contribution: (b0+b1+b2) mod 85, ranges over all 85 values
# → char4 is NEVER stable. 0% stability.
#
# Wait, but that can't be right either — when all unknown bytes are the
# same value, the sum is fixed... no, b0+b1+b2 ranges from 0 to 765,
# which covers all residues mod 85.

# Actually let me reconsider. The ENTRY stability was also about V//85^4,
# which is a different function than V mod 85. Let me think about what
# "stable" means for exit.
#
# For ENTRY, we emit leading chars and check if they're determined by
# the known bytes alone. Stability = unique leading char regardless of
# unknown bytes.
#
# For EXIT, we emit trailing chars and check if they're determined by
# the known bytes alone. Stability = unique trailing char regardless of
# unknown bytes.
#
# I just proved that char4 (V mod 85) is NOT determined by b3 alone.
# So exit 1-byte stability for char4 = 0%.
#
# This contradicts the previous finding. Let me trace the error.

print("=" * 70)
print("TRACING THE ERROR")
print("=" * 70)
print()
print("Previous claim: 'V mod 85 depends only on low byte because 0xFFFFFF00 mod 85 = 0'")
print()
print("The reasoning was: V = H + b3 where H = b0*2^24 + b1*2^16 + b2*2^8")
print("If H mod 85 = 0 for all H, then V mod 85 = b3 mod 85.")
print("But H mod 85 = 0 only when b0+b1+b2 ≡ 0 (mod 85), NOT for all H.")
print()
print("The specific value 0xFFFFFF00 has b0=b1=b2=0xFF=255, sum=765=9*85, so mod 85 = 0.")
print("That's one specific case, not a general property.")
print()

# What about 2^8 mod 85 = 1? Let me verify more carefully.
print("Double-checking: 256 mod 85 =", 256 % 85)
print("85 * 3 =", 85 * 3, "so 256 - 255 =", 256 - 255)
print("Yes, 256 mod 85 = 1. ✓")
print()
print("This means for ALL Z85 digits, V mod 85^k has a nice structure.")
print("Specifically:")
for k in range(1, 6):
    p = 85**k
    r = (2**8) % p
    r16 = (2**16) % p
    r24 = (2**24) % p
    print(f"  mod 85^{k}: 2^8≡{r}, 2^16≡{r16}, 2^24≡{r24}")
