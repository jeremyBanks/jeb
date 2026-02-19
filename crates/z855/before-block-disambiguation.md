# Before-Block Disambiguation in z855

## The Problem

When a raw escape appears at block position P (0–3), the decoder has seen P Z85
digits of the current 4-byte block before encountering the escape character. It
must recover the full 32-bit block value to output the correct bytes.

The decoder has two sources of information about this "before block":
- **P Z85 digits** — these constrain the high-order bits, defining a range of
  size 85^(5-P) for the block value
- **(4-P) known low bytes** — the first (4-P) passthrough bytes overlap with
  the low bytes of the before block, giving a congruence constraint modulo
  256^(4-P)

The question is: given these constraints, is the before-block value uniquely
determined?

## The Uniqueness Condition

The before-block is uniquely recoverable if and only if the range defined by the
Z85 digits fits within one period of the byte congruence — i.e.:

```
85^(5-D) ≤ 256^(4-P)
```

where D is the number of Z85 digits output before the escape character.

This is equivalent to: **D ≥ P + 1**.

One extra digit beyond the block position is always sufficient. This is provable
because 85 < 256: at each exponent, 85^e < 256^e, so a range of size 85^e fits
strictly inside one congruence period of size 256^e.

(Formally proved in `lean/Z855.lean` as `extended_unique_P0` through
`extended_unique_P3`, using the core lemma `unique_in_small_range`.)

If D < P+1, there can be multiple valid before-block values. The maximum number
of solutions at each (D, P):

| D (digits output) | P=1 | P=2 | P=3 |
|---|---|---|---|
| 0 | 265 | 67705 | 17M |
| 1 (natural for K=8..41) | 4 | 797 | 203909 |
| 2 (natural for K=42+) | — | 10 | 2399 |
| 3 | — | — | 29 |
| 4 (= P+1 for P=3) | — | — | **unique** |

## How Each Escape Handles This

### `;` `_` `~` (K = 5, 6, 7) — Always Unambiguous

These escapes output **exactly P+1 Z85 digits** before the escape character.
The escape character itself identifies K, so no digits are spent on length
encoding. The P+1 digits serve purely as before-block disambiguation.

Result: **always unambiguous at any position, no canonical minimum needed.**

This is the clean case and is formally proved.

### `,` (K = 4) — Always Ambiguous at Non-Zero P

The `,` escape outputs only **P digits** before the escape (not P+1). This is
one digit short of the disambiguation threshold at every non-zero P.

Result: **ambiguous at P=1,2,3.** The canonical minimum rule resolves this: the
encoder checks that the actual before-block value is the minimum among all valid
candidates, and falls back to standard Z85 if not. The decoder always selects
the minimum.

At P=0 (block-aligned), there is no before-block to recover — trivially fine.

### `|` (K ≥ 8) — Depends on K

The `|` escape uses a variable-length length prefix in base-42:
- K = 8..41: 1 prefix digit
- K = 42..1763: 2 prefix digits
- K = 1764..74087: 3 prefix digits

The total output length is `ceil(K × 5/4)` characters (same as standard z855
for K bytes), giving a **slack** of:

```
slack = ceil(K × 5/4) - prefixLen(K) - 1 - K
      = ceil(K/4) - 1   (approximately)
```

This slack can be used to pad the prefix with extra digits before `|`, pushing
D up to `prefixLen(K) + slack`. Disambiguation at position P requires D ≥ P+1.

| K range | prefixLen | slack | Max disambiguable P | Needs canonical min at |
|---|---|---|---|---|
| 8 | 1 | 0 | 0 | P=1,2,3 |
| 9–12 | 1 | 1 | 1 | P=2,3 |
| 13–16 | 1 | 2 | 2 | P=3 |
| 17–20 | 1 | 3 | 3 | never |
| 21+ | 1+ | 4+ | 3 | never |

So **canonical minimum is needed for `|` escapes when K ≤ 16 and P is large
enough that slack < P**.

Specifically:
- K=8: canonical min needed at P=1,2,3 (same exposure as `,`)
- K=9..12: canonical min needed at P=2,3
- K=13..16: canonical min needed at P=3
- K≥17: canonical min never needed (slack ≥ 3, can always emit P+1 digits)

## Summary

| Escape | K | Canonical min needed? |
|---|---|---|
| `,` | 4 | Yes, at P=1,2,3 |
| `;` | 5 | No |
| `_` | 6 | No |
| `~` | 7 | No |
| `\|` | 8 | Yes, at P=1,2,3 |
| `\|` | 9–12 | Yes, at P=2,3 |
| `\|` | 13–16 | Yes, at P=3 |
| `\|` | ≥17 | No (slack sufficient to pad prefix to P+1 digits) |

## Formal Proofs

These results are machine-verified in Lean 4 (`lean/Z855Disambiguation.lean`).
The proofs are structured to mirror the prose argument above.

### Core uniqueness lemma

A range of size `< modulus` contains at most one element of any congruence
class. If two solutions `n`, `m` exist in `[lo, lo+size)` with `n % mod = m %
mod`, their difference is `< size < mod`, yet divisible by `mod` — so it must
be zero.

```lean
theorem unique_in_small_range
    (lo size modulus : Nat) (hsize : size < modulus)
    (n m : Nat)
    (hn : lo ≤ n ∧ n < lo + size) (hm : lo ≤ m ∧ m < lo + size)
    (hcong : n % modulus = m % modulus) : n = m
```

Applied to z855: with D digits known, range = 85^(5-D); with (4-P) low bytes
known, modulus = 256^(4-P). Setting D = P+1 gives range = 85^(4-P) <
256^(4-P) (since 85 < 256 at every exponent):

```lean
example : 85^4 < 256^4 := by native_decide  -- P=0, D=1
example : 85^3 < 256^3 := by native_decide  -- P=1, D=2
example : 85^2 < 256^2 := by native_decide  -- P=2, D=3
example : 85^1 < 256^1 := by native_decide  -- P=3, D=4
```

### `,` ambiguity

With D = P (one short), range = 85^(5-P) > 256^(4-P) — multiple solutions
exist. At P=3 there are up to 29:

```lean
example : 85^2 > 256^1 := by native_decide  -- P=3, D=3: ambiguous
example : ((List.range 7225).filter (fun n => n % 256 = 42)).length = 29
        := by native_decide
```

### `|` escape slack analysis

For K < 42 (single prefix digit), the proof follows three named steps:

```lean
-- Step 1: total output chars = K + ceil(K/4)
-- (ceil(K×5/4) = K + ceil(K/4) since K is a whole number)
theorem total_eq (K : Nat) :
    (K * 5 + 3) / 4 = K + (K + 3) / 4 := by omega

-- Step 2: slack = ceil(K/4) - 2
-- (total - 1 prefix digit - 1 pipe char - K raw bytes = ceil(K/4) - 2)
theorem slack_eq (K : Nat) (h : 2 ≤ (K + 3) / 4) :
    (K * 5 + 3) / 4 - K - 2 = (K + 3) / 4 - 2 := by omega

-- Step 3a: K ≥ 17 → ceil(K/4) ≥ 5 → slack ≥ 3 → unambiguous at all P
theorem long_escape_sufficient_slack (K : Nat) (hlo : 17 ≤ K) (hhi : K < 42) :
    3 ≤ (K * 5 + 3) / 4 - K - 2 := by
  have hceil : 5 ≤ (K + 3) / 4 := by omega  -- ceil(K/4) ≥ 5
  have hslack : (K * 5 + 3) / 4 - K - 2 = (K + 3) / 4 - 2 := by omega
  omega

-- Step 3b: K < 17 → ceil(K/4) ≤ 4 → slack < 3 → canonical minimum needed at P=3
theorem long_escape_insufficient_slack (K : Nat) (hlo : 8 ≤ K) (hhi : K < 17) :
    (K * 5 + 3) / 4 - K - 2 < 3 := by
  have hceil : (K + 3) / 4 ≤ 4 := by omega  -- ceil(K/4) ≤ 4
  have hslack : (K * 5 + 3) / 4 - K - 2 = (K + 3) / 4 - 2 := by omega
  omega

-- Boundary: K=16 has slack=2 (not enough), K=17 has slack=3 (just enough)
example : (16 + 3) / 4 = 4 := by native_decide  -- ceil(16/4) = 4, slack = 2
example : (17 + 3) / 4 = 5 := by native_decide  -- ceil(17/4) = 5, slack = 3
```

The `omega` tactic handles Nat linear arithmetic automatically; the `have`
steps name the intermediate claims so the proof mirrors the argument in prose.

## Why `,` Can't Be Fixed Without Longer Output

One might ask: why not make `,` also output P+1 digits, eliminating the
canonical minimum? If you add one digit before the escape, the total output
grows by one character — violating the length invariant. To compensate, the
after-block would need one fewer digit, shifting the ambiguity problem to the
other side. There is no free lunch; the length invariant binds.

The canonical minimum is the correct solution for `,` (and for `|` with small
K): it resolves ambiguity without changing output length, at the cost of
sometimes falling back to standard Z85 encoding when the actual value isn't the
minimum.

## The Deeper Reason `;_~` Work

The key insight: for `;_~`, the escape character *identifies K*, so no bits of
the prefix need to encode length. All P+1 prefix digits are freely available for
disambiguation.

For `|`, K must be encoded in the prefix, spending digits on length that could
have gone to disambiguation. This is the fundamental cost of using a single
escape character for all K≥8. A hypothetical design with dedicated escape
characters for each K up to some limit would eliminate the canonical minimum
requirement for those K values — but at the cost of escape character budget
(z855 has only 5 non-Z85 safe characters available).