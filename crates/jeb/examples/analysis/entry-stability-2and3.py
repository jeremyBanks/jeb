#!/usr/bin/env python3
"""
Z85 — Entry boundary stability for 2-byte and 3-byte cuts.

Design doc claims:
- 2-byte entry: 89% stable
- 3-byte entry: 96% stable

Let's verify these empirically using the same mathematical approach.
"""

def is_entry_stable_2byte(b0, b1):
    """
    Check if (b0, b1) allows a stable 2-byte entry cut.
    
    Stable = the leading 2 Z85 characters are determined by b0, b1 alone.
    
    For 2-byte entry [b0 b1 | unknown unknown]:
    - V ranges [b0*2^24 + b1*2^16, b0*2^24 + b1*2^16 + 2^16 - 1]
    - Width: 2^16 = 65,536
    - We need to check chars 0 and 1: floor(V / 85^4) and floor(V / 85^3) mod 85
    
    Actually, it's easier to just check if the first two chars are constant.
    char0 = V // 85^4
    char1 = (V // 85^3) % 85
    """
    v_min = (b0 << 24) | (b1 << 16)
    v_max = (b0 << 24) | (b1 << 16) | 0xFFFF
    
    # Check if both leading chars are constant across the range
    c0_min = v_min // (85**4)
    c0_max = v_max // (85**4)
    
    c1_min = (v_min // (85**3)) % 85
    c1_max = (v_max // (85**3)) % 85
    
    return (c0_min == c0_max) and (c1_min == c1_max)

def is_entry_stable_3byte(b0, b1, b2):
    """
    Check if (b0, b1, b2) allows a stable 3-byte entry cut.
    
    For 3-byte entry [b0 b1 b2 | unknown]:
    - V ranges [b0*2^24 + b1*2^16 + b2*2^8, b0*2^24 + b1*2^16 + b2*2^8 + 255]
    - Width: 256
    - We need chars 0, 1, 2 constant
    """
    v_min = (b0 << 24) | (b1 << 16) | (b2 << 8)
    v_max = (b0 << 24) | (b1 << 16) | (b2 << 8) | 0xFF
    
    # Check if first three chars are constant
    c0_min = v_min // (85**4)
    c0_max = v_max // (85**4)
    
    c1_min = (v_min // (85**3)) % 85
    c1_max = (v_max // (85**3)) % 85
    
    c2_min = (v_min // (85**2)) % 85
    c2_max = (v_max // (85**2)) % 85
    
    return (c0_min == c0_max) and (c1_min == c1_max) and (c2_min == c2_max)

def analyze_2byte():
    print("=" * 70)
    print("2-BYTE ENTRY (b0,b1 known, b2/b3 unknown)")
    print("=" * 70)
    print()
    print("Testing all 65,536 possible (b0,b1) pairs...")
    print()
    
    stable_count = 0
    total = 256 * 256
    
    for b0 in range(256):
        for b1 in range(256):
            if is_entry_stable_2byte(b0, b1):
                stable_count += 1
    
    percent = 100.0 * stable_count / total
    
    print(f"Result: {stable_count}/{total} pairs have stable leading 2 chars")
    print(f"        ({percent:.1f}% stable)")
    print()
    
    return percent

def analyze_3byte():
    print("=" * 70)
    print("3-BYTE ENTRY (b0,b1,b2 known, b3 unknown)")
    print("=" * 70)
    print()
    print("Testing all 16,777,216 possible (b0,b1,b2) triplets...")
    print("(This will take a moment...)")
    print()
    
    stable_count = 0
    total = 256 * 256 * 256
    
    # Progress indicator
    for b0 in range(256):
        if b0 % 32 == 0:
            print(f"  Progress: {b0}/256 ({100*b0/256:.0f}%)...", end='\r')
        for b1 in range(256):
            for b2 in range(256):
                if is_entry_stable_3byte(b0, b1, b2):
                    stable_count += 1
    
    print()  # Clear progress line
    percent = 100.0 * stable_count / total
    
    print(f"Result: {stable_count:,}/{total:,} triplets have stable leading 3 chars")
    print(f"        ({percent:.1f}% stable)")
    print()
    
    return percent

if __name__ == "__main__":
    print()
    print("Z85 Entry Boundary Stability — 2-byte and 3-byte cuts")
    print("=" * 70)
    print()
    
    pct_2 = analyze_2byte()
    pct_3 = analyze_3byte()
    
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print()
    print(f"  1-byte entry: 68.0% stable (from previous analysis)")
    print(f"  2-byte entry: {pct_2:.1f}% stable")
    print(f"  3-byte entry: {pct_3:.1f}% stable")
    print()
    print("Design doc claims: 68%, 89%, 96%")
    print()
    
    if abs(pct_2 - 89.0) < 1.0:
        print("✓ 2-byte claim verified")
    else:
        print(f"✗ 2-byte mismatch: expected ~89%, got {pct_2:.1f}%")
    
    if abs(pct_3 - 96.0) < 1.0:
        print("✓ 3-byte claim verified")
    else:
        print(f"✗ 3-byte mismatch: expected ~96%, got {pct_3:.1f}%")
    print()
