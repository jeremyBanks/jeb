#!/usr/bin/env python3
"""
Z85 — Opportunistic zero-padding analysis for exit boundaries.

The opportunistic zero-padding strategy:
- For an exit boundary, check if encoding the boundary bytes with the actual
  following bytes produces the same Z85 characters as encoding them with zeros.
- If yes: the cut is "self-signaling" — decoder can assume zero-padding and
  uniquely recover the boundary bytes. Zero bit cost.
- If no: skip the cut, use block-aligned exit or extend the raw section.

This script analyzes which byte values allow zero-padding cuts.
"""

def z85_encode_block(b0, b1, b2, b3):
    """Encode 4 bytes as 5 Z85 characters (returns list of 5 digit values 0-84)."""
    v = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    digits = []
    for _ in range(5):
        digits.append(v % 85)
        v //= 85
    return list(reversed(digits))  # Z85 is big-endian (most significant first)

def allows_zero_padding_exit(known_bytes, actual_unknown_bytes):
    """
    Check if this specific 4-byte block allows a zero-padding exit cut.
    
    The encoder checks: do the actual unknown bytes encode the same trailing
    characters as if they were zeros? If yes, the decoder can assume zeros
    and uniquely recover the boundary bytes.
    
    Args:
        known_bytes: tuple of K bytes at END of block (1-3 bytes)
        actual_unknown_bytes: tuple of (4-K) actual bytes at START of block
    
    Returns:
        True if encoding actual block matches encoding zero-padded block
    """
    K = len(known_bytes)
    unknown_count = len(actual_unknown_bytes)
    assert K + unknown_count == 4, "Must total 4 bytes"
    
    # Build the zero-padded block: zeros for unknown, then known bytes
    zero_padded = [0] * unknown_count + list(known_bytes)
    zero_encoding = z85_encode_block(*zero_padded)
    
    # Build the actual block
    actual_block = list(actual_unknown_bytes) + list(known_bytes)
    actual_encoding = z85_encode_block(*actual_block)
    
    # Compare only the trailing K+1 characters (those affected by boundary)
    # For 1-byte exit: trailing 2 chars (indices 3,4)
    # For 2-byte exit: trailing 3 chars (indices 2,3,4)
    # For 3-byte exit: trailing 4 chars (indices 1,2,3,4)
    trailing_start = 5 - (K + 1)
    
    return zero_encoding[trailing_start:] == actual_encoding[trailing_start:]

def analyze_1byte_exits():
    """Analyze what fraction of 1-byte exit scenarios allow zero-padding."""
    print("=" * 70)
    print("1-BYTE EXIT (b3 known, b0/b1/b2 unknown)")
    print("=" * 70)
    print()
    print("Testing random 4-byte blocks to see what fraction allow exit cuts...")
    print("For each block, checking if [b0,b1,b2,b3] encodes the same trailing")
    print("characters as [0,0,0,b3].")
    print()
    
    import random
    random.seed(42)
    
    sample_size = 10000
    stable_count = 0
    
    examples_yes = []
    examples_no = []
    
    for _ in range(sample_size):
        b0, b1, b2, b3 = [random.randint(0, 255) for _ in range(4)]
        if allows_zero_padding_exit((b3,), (b0, b1, b2)):
            stable_count += 1
            if len(examples_yes) < 5:
                examples_yes.append((b0, b1, b2, b3))
        else:
            if len(examples_no) < 5:
                examples_no.append((b0, b1, b2, b3))
    
    percent = 100.0 * stable_count / sample_size
    
    print(f"Result: {stable_count}/{sample_size} blocks allow zero-padding exits")
    print(f"({percent:.2f}% of random blocks)")
    print()
    
    if examples_yes:
        print(f"Example blocks that ALLOW exit cut:")
        for block in examples_yes:
            print(f"  {block}")
        print()
    
    if examples_no:
        print(f"Example blocks that DISALLOW exit cut:")
        for block in examples_no:
            print(f"  {block}")
        print()
    
    return percent

def analyze_2byte_exits():
    """Analyze what fraction of 2-byte exit scenarios allow zero-padding."""
    print("=" * 70)
    print("2-BYTE EXIT (b2/b3 known, b0/b1 unknown)")
    print("=" * 70)
    print()
    print("Testing random 4-byte blocks...")
    print()
    
    import random
    random.seed(43)
    
    sample_size = 10000
    stable_count = 0
    
    for _ in range(sample_size):
        b0, b1, b2, b3 = [random.randint(0, 255) for _ in range(4)]
        if allows_zero_padding_exit((b2, b3), (b0, b1)):
            stable_count += 1
    
    percent = 100.0 * stable_count / sample_size
    print(f"Result: {stable_count}/{sample_size} blocks allow zero-padding exits")
    print(f"({percent:.2f}% of random blocks)")
    print()
    
    return percent

def analyze_3byte_exits():
    """Analyze what fraction of 3-byte exit scenarios allow zero-padding."""
    print("=" * 70)
    print("3-BYTE EXIT (b1/b2/b3 known, b0 unknown)")
    print("=" * 70)
    print()
    print("Testing random 4-byte blocks...")
    print()
    
    import random
    random.seed(44)
    
    sample_size = 10000
    stable_count = 0
    
    for _ in range(sample_size):
        b0, b1, b2, b3 = [random.randint(0, 255) for _ in range(4)]
        if allows_zero_padding_exit((b1, b2, b3), (b0,)):
            stable_count += 1
    
    percent = 100.0 * stable_count / sample_size
    print(f"Result: {stable_count}/{sample_size} blocks allow zero-padding exits")
    print(f"({percent:.2f}% of random blocks)")
    print()
    
    return percent

if __name__ == "__main__":
    print()
    print("Z85 Opportunistic Zero-Padding Analysis")
    print("=" * 70)
    print()
    print("Strategy: Only allow exit cuts when the ACTUAL boundary bytes")
    print("encode the same trailing characters as if unknown bytes were zeros.")
    print()
    print("The encoder checks each potential exit cut:")
    print("  1. Encode actual block [b0,b1,b2,b3]")
    print("  2. Encode zero-padded block [0,0,0,b3] (for 1-byte exit)")
    print("  3. If trailing chars match: allow cut (decoder assumes zeros)")
    print("  4. If trailing chars differ: skip cut, try different position")
    print()
    
    pct_1byte = analyze_1byte_exits()
    pct_2byte = analyze_2byte_exits()
    pct_3byte = analyze_3byte_exits()
    
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print()
    print(f"  1-byte exits: {pct_1byte:.2f}% of blocks allow zero-padding")
    print(f"  2-byte exits: {pct_2byte:.2f}% of blocks allow zero-padding")
    print(f"  3-byte exits: {pct_3byte:.2f}% of blocks allow zero-padding")
    print()
    print("=" * 70)
    print("CONCLUSION")
    print("=" * 70)
    print()
    print("Opportunistic zero-padding:")
    print("  ✓ Zero bit cost (self-signaling when it works)")
    print("  ✓ Encoder complexity (checking each potential cut)")
    print("  ✗ Only works for a fraction of byte combinations")
    print()
    print("The encoder tries exit cuts opportunistically:")
    print("  - If the actual bytes allow it: take the cut, zero cost")
    print("  - If not: use block-aligned exit or extend raw section")
    print()
    print("This is a practical trade-off: eliminate disambiguation bits")
    print("at the cost of restricting which byte patterns allow mid-block cuts.")
    print()
