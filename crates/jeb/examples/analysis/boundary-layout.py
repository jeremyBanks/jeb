#!/usr/bin/env python3
"""
Z85 Extended — Concrete Boundary Layout Analysis

Work through specific examples of how a raw section with mid-block
cuts at entry and/or exit would be laid out in the output stream.
"""

Z85 = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#"

def z85_encode(b0, b1, b2, b3):
    v = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    chars = []
    for _ in range(5):
        chars.append(chr(Z85[v % 85]))
        v //= 85
    return ''.join(reversed(chars))

def z85_partial(bytes_list):
    """Encode partial block (1-3 bytes) as Z85."""
    padded = list(bytes_list) + [0] * (4 - len(bytes_list))
    full = z85_encode(*padded)
    n_chars = len(bytes_list) + 1  # n bytes → n+1 chars
    return full[:n_chars]

print("=" * 70)
print("CONCRETE BOUNDARY LAYOUT EXAMPLES")
print("=" * 70)

# ================================================================
# Example 1: Block-aligned entry and exit (simplest case)
# ================================================================
print("""
EXAMPLE 1: Block-aligned entry and exit
========================================
Input: 12 bytes, bytes 4-11 are ASCII-safe, bytes 0-3 and 12-15 are not.

Byte layout:    [b0 b1 b2 b3] [b4 b5 b6 b7 b8 b9 b10 b11] [b12 b13 b14 b15]
Block layout:   [---block 0--] [------raw section---------] [---block 3------]
Z85 chars:      c0c1c2c3c4     ESC + 8 raw + padding        c15c16c17c18c19

Standard Z85 for 16 bytes = 20 chars.
Our encoding: 5 (block 0) + ? (raw section) + 5 (block 3) = need 10 chars for raw.
Raw section: 8 bytes raw + 1 escape + 1 padding/length = 10 chars. ✓
Budget: 10 - 8 = 2 chars for escape + overhead.
""")

# ================================================================
# Example 2: Mid-block entry, block-aligned exit
# ================================================================
print("""
EXAMPLE 2: Mid-block entry (cut after 1 byte), block-aligned exit
==================================================================
Input: 15 bytes. Byte 0 is not safe. Bytes 1-11 are safe. Bytes 12-15 not safe.

Standard Z85 for 16 bytes = 20 chars.
Block 0: [b0 | b1 b2 b3]  — we want b0 as Z85, b1-b3 as raw
                             entry cut after 1 byte

If b0's char0 is STABLE (68% chance):
  Emit char0 for b0 (1 char, BE leading)
  Then escape char (1 char)
  Then raw: b1 b2 b3 b4 b5 b6 b7 b8 b9 b10 b11 (11 bytes)
  Then padding to fill remaining budget
  Then block 3: c15-c19 (5 chars)
  
  Char positions used:
    Position 0: char0 (partial Z85 for b0)
    Position 1: ESCAPE
    Positions 2-12: raw bytes (11 bytes)
    Positions 13-14: padding (2 chars)
    Positions 15-19: block 3 (standard Z85)
    Total: 20 chars ✓

  Budget for raw section: 20 - 5 (block 3) - 1 (char0) = 14 positions
  Used: 1 escape + 11 raw = 12. Remaining: 2 for padding/length/disambig.
  
If b0's char0 is NOT stable (32% chance):
  Need 2 bits of disambiguation for b0.
  Same layout but 1 of the 2 padding chars carries disambig bits.
  Budget: 14 - 1 (escape) - 11 (raw) - 1 (disambig) = 1 for padding. Still works.
""")

# ================================================================
# Example 3: Block-aligned entry, mid-block exit
# ================================================================
print("""
EXAMPLE 3: Block-aligned entry, mid-block exit (cut before last 1 byte)
========================================================================
Input: 15 bytes. Bytes 0-3 not safe. Bytes 4-14 safe. Byte 15 not safe.

Block 0: [b0 b1 b2 b3] — standard Z85
Raw: b4-b14 (11 bytes)
Exit block: [b12 b13 b14 | b15] — b12-b14 were raw, b15 needs Z85

For exit: b15 is the LAST byte of the block. Its information is in
the trailing Z85 char (char4 of the exit block).
  char4 = V mod 85, where V includes b15 in the low byte.
  This is ALWAYS determined by b15 alone (100% stable).
  
  Layout:
    Positions 0-4: block 0 (standard Z85)
    Position 5: ESCAPE
    Positions 6-16: raw bytes b4-b14 (11 bytes)
    Position 17: trailing Z85 char for b15
    Positions 18-19: padding (2 chars)
    Total: 20 chars ✓
    
  But wait — does the trailing char go in position 17? 
  In standard Z85, block 3 occupies positions 15-19.
  The trailing char (char4 of that block) would be at position 19.
  
  Position invariant says Z85 chars must appear at the SAME positions
  as standard Z85 would produce. So char4 of block 3 goes at position 19.
  
  Revised layout:
    Positions 0-4: block 0 (standard Z85)
    Position 5: ESCAPE (or part of escape sequence)
    Positions 6-16: raw bytes b4-b14 (11 bytes)
    Positions 17-18: padding/overhead
    Position 19: trailing Z85 char for b15
    Total: 20 chars ✓
    
  Hmm, but this means the Z85 char at position 19 might NOT match what
  standard Z85 would produce there, because it only encodes b15 (not
  the full block). This is the exception Jeremy mentioned — the cut-off
  block's chars may differ from standard Z85, and that's OK as long as
  they're consistent with Z85 encoding of the included bits.
""")

# ================================================================
# Example 4: Mid-block entry AND exit
# ================================================================
print("""
EXAMPLE 4: Mid-block entry AND exit
=====================================
Input: 20 bytes. Byte 0 not safe. Bytes 1-18 safe. Byte 19 not safe.

Standard Z85 for 20 bytes = 25 chars.
Entry: cut after byte 0 (1 byte Z85, rest raw)
Exit: cut before byte 19 (raw stops, 1 byte Z85)

  Entry: 1 leading Z85 char for b0 (position 0)
  Escape: 1 char (position 1)  
  Raw: bytes 1-18 (18 bytes, positions 2-19)
  Trailing Z85 char for b19 (position 24)
  Padding/length/disambig: positions 20-23 (4 chars)
  
  Budget: 25 - 18 (raw) - 1 (entry char) - 1 (exit char) = 5 chars
  Need: 1 escape + possibly 1 length + possibly 2 disambig = 2-4
  Remaining: 1-3 chars padding. Comfortable.

For 8 raw bytes (minimum interesting case):
  Standard Z85 for 12 bytes = 15 chars.
  Entry: 1 char (pos 0)
  Escape: 1 char (pos 1)
  Raw: 8 bytes (pos 2-9)
  Exit: 1 char (pos 14)
  Overhead: positions 10-13 (4 chars)
  Budget: 15 - 8 - 1 - 1 = 5 chars for escape + length + disambig + padding
  Works comfortably.

For 4 raw bytes (tight case):
  Standard Z85 for 8 bytes = 10 chars.
  Entry: 1 char (pos 0)
  Escape: 1 char (pos 1)
  Raw: 4 bytes (pos 2-5)
  Exit: 1 char (pos 9)
  Overhead: positions 6-8 (3 chars)
  Budget: 10 - 4 - 1 - 1 = 4 chars for escape + length + disambig + padding
  Still works! (escape=1, length implied by escape choice, disambig=1-2, padding=rest)
""")

# ================================================================
# Example 5: Very short raw section (4 bytes, block-aligned)
# ================================================================
print("""
EXAMPLE 5: Minimal case — 4 raw bytes, block-aligned
======================================================
Input: 4 bytes, all ASCII-safe.

Standard Z85: 5 chars.
Our encoding: 1 escape + 4 raw = 5 chars. Exactly fits!
Budget: 0 chars for anything else.

This is the tightest case. No room for:
- Length encoding (length must be implied by the escape char choice)
- Disambiguation (mid-block cuts impossible)
- Padding

So with 1 escape char: we can do block-aligned 4-byte raw sections.
With 2 escape chars: same, but we get 1 bit of info from the choice.
""")

# ================================================================
# Summary table
# ================================================================
print("=" * 70)
print("BUDGET SUMMARY FOR VARIOUS CONFIGURATIONS")
print("=" * 70)
print()
print(f"{'Raw bytes':>10} {'Entry cut':>10} {'Exit cut':>10} {'Total chars':>12} {'Budget':>7} {'Needs':>30}")
print("-" * 85)

configs = [
    (4, 'aligned', 'aligned', "1 esc"),
    (4, 'mid-1', 'aligned', "1 esc + 1 disambig"),
    (4, 'aligned', 'mid-1', "1 esc + 1 disambig"),
    (4, 'mid-1', 'mid-1', "1 esc + 2 disambig"),
    (8, 'aligned', 'aligned', "1 esc"),
    (8, 'mid-1', 'aligned', "1 esc + 1 disambig"),
    (8, 'mid-1', 'mid-1', "1 esc + 2 disambig"),
    (12, 'mid-1', 'mid-1', "1 esc + 1 len + 2 disambig"),
    (20, 'mid-1', 'mid-1', "1 esc + 1 len + 2 disambig"),
    (100, 'mid-1', 'mid-1', "1 esc + 1 len + 2 disambig"),
]

for raw_n, entry, exit_, needs in configs:
    # Entry bytes that become Z85
    entry_z85 = 1 if 'mid' in entry else 0
    exit_z85 = 1 if 'mid' in exit_ else 0
    total_input = raw_n + entry_z85 + exit_z85
    # Round up to block boundary for total input
    # Actually: total chars = ceil(total_input * 5 / 4) if we're replacing
    # a segment of the standard Z85 stream
    # Simpler: budget = ceil(raw_n * 5 / 4) - raw_n for the raw portion
    # Plus the entry/exit Z85 chars come from their own block budgets
    
    # Total standard Z85 chars for all bytes involved
    # This includes the entry and exit boundary blocks
    total_bytes_in_blocks = raw_n + (4 if 'mid' in entry else 0) + (4 if 'mid' in exit_ else 0)
    total_z85_chars = -(-total_bytes_in_blocks * 5 // 4)
    
    # Chars consumed: entry Z85 chars + raw bytes + exit Z85 chars
    consumed = entry_z85 + raw_n + exit_z85
    
    budget = total_z85_chars - consumed
    
    # But we also need to account for the rest of the entry/exit blocks
    # being standard Z85 chars (which don't cost budget)
    # This is getting complicated — let me just use the simple formula
    
    simple_budget = -(-raw_n * 5 // 4) - raw_n
    
    print(f"{raw_n:>10} {entry:>10} {exit_:>10} {-(-raw_n * 5 // 4):>12} {simple_budget:>7} {needs:>30}")
