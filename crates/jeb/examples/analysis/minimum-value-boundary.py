#!/usr/bin/env python3
"""
Z85 — Minimum-value boundary analysis.

Test the minimum-value constraint: a boundary cut is valid if the actual
boundary bytes produce the MINIMUM possible Z85 encoding given the known bytes.

Compare to zero-padding approach.
"""

def z85_encode_block(b0, b1, b2, b3):
    """Encode 4 bytes as 5 Z85 characters (returns list of 5 digit values 0-84)."""
    v = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    digits = []
    for _ in range(5):
        digits.append(v % 85)
        v //= 85
    return list(reversed(digits))  # Z85 is big-endian

def check_minimum_value_entry_1byte(b0, b1, b2, b3):
    """
    Check if [b0,b1,b2,b3] produces minimum encoding for 1-byte entry cut.
    
    Known: b0
    Unknown: b1, b2, b3
    
    The minimum V for given b0 occurs when b1=b2=b3=0.
    Check if actual encoding matches minimum encoding.
    """
    actual = z85_encode_block(b0, b1, b2, b3)
    minimum = z85_encode_block(b0, 0, 0, 0)
    
    # For 1-byte entry, we emit the leading character (char 0)
    # It's valid if actual[0] == minimum[0]
    return actual[0] == minimum[0]

def check_minimum_value_exit_1byte(b0, b1, b2, b3):
    """
    Check if [b0,b1,b2,b3] produces minimum encoding for 1-byte exit cut.
    
    Known: b3
    Unknown: b0, b1, b2
    
    The minimum V for given b3 occurs when b0=b1=b2=0.
    Check if actual encoding matches minimum encoding.
    """
    actual = z85_encode_block(b0, b1, b2, b3)
    minimum = z85_encode_block(0, 0, 0, b3)
    
    # For 1-byte exit, we emit the trailing character (char 4)
    # It's valid if actual[4] == minimum[4]
    return actual[4] == minimum[4]

def analyze_entry_1byte():
    """Test minimum-value constraint for 1-byte entry cuts."""
    print("=" * 70)
    print("1-BYTE ENTRY (Minimum-Value Constraint)")
    print("=" * 70)
    print()
    
    import random
    random.seed(42)
    
    sample_size = 10000
    valid_count = 0
    
    for _ in range(sample_size):
        b0, b1, b2, b3 = [random.randint(0, 255) for _ in range(4)]
        if check_minimum_value_entry_1byte(b0, b1, b2, b3):
            valid_count += 1
    
    percent = 100.0 * valid_count / sample_size
    print(f"Result: {valid_count}/{sample_size} blocks satisfy minimum-value")
    print(f"({percent:.2f}% viable)")
    print()
    
    return percent

def analyze_exit_1byte():
    """Test minimum-value constraint for 1-byte exit cuts."""
    print("=" * 70)
    print("1-BYTE EXIT (Minimum-Value Constraint)")
    print("=" * 70)
    print()
    
    import random
    random.seed(43)
    
    sample_size = 10000
    valid_count = 0
    
    for _ in range(sample_size):
        b0, b1, b2, b3 = [random.randint(0, 255) for _ in range(4)]
        if check_minimum_value_exit_1byte(b0, b1, b2, b3):
            valid_count += 1
    
    percent = 100.0 * valid_count / sample_size
    print(f"Result: {valid_count}/{sample_size} blocks satisfy minimum-value")
    print(f"({percent:.2f}% viable)")
    print()
    
    return percent

if __name__ == "__main__":
    print()
    print("Z85 Minimum-Value Boundary Analysis")
    print("=" * 70)
    print()
    print("Strategy: A boundary cut is valid if the actual boundary bytes")
    print("produce the MINIMUM possible Z85 encoding for the known bytes.")
    print()
    
    entry_pct = analyze_entry_1byte()
    exit_pct = analyze_exit_1byte()
    
    print("=" * 70)
    print("COMPARISON WITH ZERO-PADDING")
    print("=" * 70)
    print()
    print("From previous analysis (zero-padding):")
    print("  1-byte entry: ~68.0% stable (natural stability)")
    print("  1-byte exit:  ~0.02% viable")
    print()
    print("Minimum-value constraint:")
    print(f"  1-byte entry: {entry_pct:.2f}% viable")
    print(f"  1-byte exit:  {exit_pct:.2f}% viable")
    print()
    
    if abs(exit_pct - 0.02) < 0.5:
        print("✓ Exit rates match — minimum-value ≈ zero-padding for exits")
    
    print()
    print("Key insight: Minimum-value is less disruptive because it's a")
    print("property of the encoding itself (is this the minimum?), not an")
    print("assumption about unknown bytes (assume they're zero).")
    print()
