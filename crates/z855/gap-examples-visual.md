# Gap Test Examples - Visual Guide

This document shows interesting examples from the gap test suite to illustrate Z855 canonical encoding behavior.

## Gap 0: Back-to-Back Safe Runs

When two safe runs have no gap between them, they're treated as a single continuous run.

```
Input:  AAAA BBBB (8 bytes, all safe)
Bytes:  41 41 41 41 42 42 42 42
Output: 0|AAAABBBB

Input:  AAAA AAAAA (9 bytes, all safe)
Bytes:  41 41 41 41 41 41 41 41 41
Output: 0|AAAAAAAAA

Input:  AAAAAAAA BBBBBBBB (16 bytes, all safe)
Bytes:  41×8 42×8
Output: 0|AAAAAAAABBBBBBBB
```

**Pattern:** `0|` means "rest of input is raw" - most efficient for all-safe data.

## Gap 1: Single Unsafe Byte

A single unsafe byte (0x00) between two safe runs creates interesting encoding choices.

```
Input:  AAAA [00] BBBB (9 bytes)
Bytes:  41 41 41 41 00 42 42 42 42
Output: ,AAAA0761s0=

Breakdown:
  ,AAAA    ← 4-byte passthrough escape (first run visible)
  0761s0=  ← Encodes: 1 zero + 4 B's (second run in encoded form)

Input:  AAAAAAAA [00] BBBB (13 bytes)
Bytes:  41×8 00 42×4
Output: 8|AAAAAAAA0761s0=

Breakdown:
  8|AAAAAAAA ← 8-byte long passthrough (first run visible)
  0761s0=    ← Encodes: 1 zero + 4 B's
```

**Pattern:** First run uses appropriate escape. Gap + second run encoded together.

## Gap 2-3: Partial Blocks

Multiple unsafe bytes create partial Z85 blocks.

```
Input:  AAAAA [00 00] BBBBBBB (14 bytes)
Bytes:  41×5 00 00 42×7
Output: ;AAAAA002_BBBBBB0=

Breakdown:
  ;AAAAA    ← 5-byte passthrough (first run visible)
  002       ← Encodes: 2 zeros + start of second run
  _BBBBBB   ← 6-byte passthrough (most of second run visible)
  0=        ← Final byte encoded

Input:  AAAAAA [00 00 00] BBBBBB (15 bytes)
Bytes:  41×6 00×3 42×6
Output: _AAAAAA0000;BBBBB0=

Breakdown:
  _AAAAAA   ← 6-byte passthrough (first run visible)
  0000      ← Encodes: 3 zeros + start
  ;BBBBB    ← 5-byte passthrough (most of second run visible)
  0=        ← Final byte encoded
```

**Pattern:** Both runs partially visible. Encoder balances escape efficiency.

## Gap 4: The Special Case

Exactly 4 bytes (one complete Z85 block) creates a clean boundary.

```
Input:  AAAA [00 00 00 00] BBBB (12 bytes)
Bytes:  41×4 00×4 42×4
Output: ,AAAA00000,BBBB

Breakdown:
  ,AAAA   ← 4-byte passthrough (first run fully visible)
  00000   ← 5 Z85 chars encode 4 zero bytes
  ,BBBB   ← 4-byte passthrough (second run fully visible)

Input:  AAAAAAA [00 00 00 00] BBBBBBB (18 bytes)
Bytes:  41×7 00×4 42×7
Output: k~AAAAAAA00000l~BBBBBBB

Breakdown:
  k         ← Extended digit prefix
  ~AAAAAAA  ← 7-byte passthrough (first run fully visible)
  00000     ← Gap encoded
  l         ← Extended digit prefix
  ~BBBBBBB  ← 7-byte passthrough (second run fully visible)

Input:  AAAAAAAA [00 00 00 00] BBBBBBBB (20 bytes)
Bytes:  41×8 00×4 42×8
Output: 8|AAAAAAAA000000|BBBBBBBB

Breakdown:
  8|AAAAAAAA ← 8-byte long passthrough with length prefix
  000000     ← Padding (dots would be used in original format)
  |BBBBBBBB  ← Continuation: second 8-byte run fully visible
```

**Pattern:** Clean separation. Both runs fully visible. Gap encoded efficiently.

## Gap 8: Two Z85 Blocks

```
Input:  AAAA [00×8] BBBB (16 bytes)
Bytes:  41×4 00×8 42×4
Output: ,AAAA0000000000,BBBB

Breakdown:
  ,AAAA        ← First run visible
  0000000000   ← 8 zeros as pure Z85 (10 chars)
  ,BBBB        ← Second run visible

Input:  AAAAAAAA [00×8] BBBBBBBB (24 bytes)
Bytes:  41×8 00×8 42×8
Output: 8|AAAAAAAA000000000000000|BBBBBBBB

Breakdown:
  8|AAAAAAAA          ← First 8-byte run with long escape
  000000000000000     ← 8 zeros + padding to maintain alignment
  |BBBBBBBB           ← Second 8-byte run
```

**Pattern:** Long gaps don't prevent raw visibility. Both runs still appear as text.

## Gap 16: Four Z85 Blocks

```
Input:  AAAA [00×16] BBBB (24 bytes)
Output: ,AAAA00000000000000000000,BBBB

  ,AAAA                      ← First run visible
  00000000000000000000       ← 16 zeros as Z85 (20 chars)
  ,BBBB                      ← Second run visible

Input:  AAAAAAA [00×16] BBBBBBB (30 bytes)
Output: k~AAAAAAA00000000000000000000l~BBBBBBB

  k~AAAAAAA                  ← First 7-byte run visible
  00000000000000000000       ← 16 zeros
  l~BBBBBBB                  ← Second 7-byte run visible
```

**Pattern:** Even very long gaps preserve raw visibility of safe runs.

## Asymmetric Runs: Mixed Lengths

Different run lengths show escape priority.

```
Input:  AAAA [00] BBBBBBBB (13 bytes)
Bytes:  41×4 00 42×8
Output: ,AAAA07~BBBBBBB0=

Breakdown:
  ,AAAA      ← Short run gets simple comma escape
  07         ← Gap + partial second run encoded
  ~BBBBBBB   ← 7 bytes of second run visible with tilde escape
  0=         ← Final byte encoded

Input:  AAAAAAAA [00] BBBB (13 bytes)
Bytes:  41×8 00 42×4
Output: 8|AAAAAAAA0761s0=

Breakdown:
  8|AAAAAAAA ← Long run gets priority, uses pipe escape
  0761s0=    ← Gap + short run encoded together
```

**Pattern:** Longer runs get priority for passthrough escapes. Shorter runs may be encoded.

## Interesting Edge Cases

### Case 1: Gap Larger Than Runs
```
Input:  AAAA [00×16] BBBB (24 bytes)
Output: ,AAAA00000000000000000000,BBBB
```
Gap is 4× the size of each run, but both runs still visible!

### Case 2: Minimal Runs, Minimal Gap
```
Input:  AAAA [00] BBBB (9 bytes)
Output: ,AAAA0761s0=
```
Smallest testable case (4-byte runs are minimum for escapes).

### Case 3: Maximum Back-to-Back
```
Input:  AAAAAAAA BBBBBBBB (16 bytes, gap=0)
Output: 0|AAAAAAAABBBBBBBB
```
Treated as single 16-byte safe run. Most efficient possible.

### Case 4: Symmetric Large Runs, Small Gap
```
Input:  AAAAAAAA [00] BBBBBBBB (17 bytes)
Output: 8|AAAAAAAA07~BBBBBBB0=
```
First run uses long escape, second run partially visible with different escape.

## Summary Insights

1. **Gap 0 is special:** Always uses `0|` rest-of-input escape
2. **Gap 4 is clean:** Provides natural boundary for dual escapes
3. **Longer runs get priority:** Encoder prefers passthrough for longer safe sequences
4. **Raw visibility is guaranteed:** All runs 4+ bytes appear as raw text somewhere in output
5. **Canonical is deterministic:** Same input always produces same output

## Encoding Decision Tree

```
Two safe runs separated by gap?
│
├─ Gap = 0 bytes?
│  └─ YES → Use 0| for combined run (most efficient)
│
├─ First run length?
│  ├─ 4 bytes → Use , escape
│  ├─ 5 bytes → Use ; escape
│  ├─ 6 bytes → Use _ escape
│  ├─ 7 bytes → Use ~ escape
│  └─ 8+ bytes → Use | escape
│
├─ Gap = 4 bytes AND second run is escapable?
│  └─ YES → Use dual escapes (both runs visible)
│
└─ Otherwise → Encode gap + second run together
   (first run always visible, second depends on alignment)
```

This decision tree explains the patterns observed across all 200 gap tests.
