# B1032 Correctness Proof

## Overview

B1032 is a bijective encoding from non-negative integers to alphanumeric strings with the property that small values (0-9999) encode as familiar decimal while larger values use base32 for density.

The key challenge is **ambiguity**: a 4-character base32 token like "1234" could be confused with decimal 1234. The encoding solves this by skipping base32 values that would produce all-digit tokens.

## Definitions

- **B** = 32⁴ = 1,048,576 (number of 4-digit base32 tokens)
- **C** = 304,426 (base32 value of "999A", the start of the uninterrupted identity region)
- **Alphabet** = `0123456789ABCDEFGHIJKLMNOPQRSTUV` (32 symbols)
- **All-digit token**: A base32 token where all characters are in `0-9`
- **Good value**: A value v in [0, B) whose 4-digit padded base32 representation contains at least one letter
- **Bad value**: A value v in [0, B) whose 4-digit padded base32 representation is all-digits

## Lemma 1: Exactly 10,000 bad values exist in [0, B)

**Proof**: A 4-digit base32 token is all-digits iff each digit is in {0,1,2,...,9}. There are 10 choices per digit, so 10⁴ = 10,000 all-digit tokens. Each corresponds to exactly one value in [0, B). ∎

## Lemma 2: No bad values exist in [C, B)

**Proof**: C = 304,426 encodes to "999A". For v ≥ C:
- The first digit is at least 9 (since C/32³ = 9.27...)
- If the first digit is 9, the remaining value is at least 304,426 - 9×32³ = 9,194
- This means subsequent digits cannot all be ≤ 9 while staying in [C, B)

More directly: "999A" through "VVVV" all contain at least one letter by construction (either the first digit is ≥ 10, or the later digits push past 9999). ∎

## Lemma 3: The encoding covers all non-negative integers without gaps

**Encoding scheme**:
1. If n ≤ 9999: encode as decimal string (no padding)
2. If n ≥ C: encode as base32 (identity mapping)
3. If 10000 ≤ n < C: encode as the (n - 10000)-th good value in [0, B)

**Proof of coverage**:

- Range [0, 9999]: 10,000 integers → 10,000 decimal strings ✓
- Range [10000, C-1]: This is C - 10000 = 294,426 integers
  - Number of good values in [0, B) = B - 10000 = 1,038,576
  - We use the first 294,426 of these (indices 0 to 294,425)
- Range [C, ∞): Identity mapping to base32 ✓

The transition is seamless:
- n = 9999 → "9999" (decimal)
- n = 10000 → 0th good value → "000A" (first 4-char token with a letter)
- n = C-1 = 304425 → 294425th good value → "998V" (last before identity)
- n = C = 304426 → "999A" (identity begins)

∎

## Theorem 1: Bijectivity

**Claim**: The encoding is a bijection between ℕ and valid B1032 strings.

**Proof**:

*Injectivity* (different integers → different strings):
- Integers 0-9999 map to distinct decimal strings by standard decimal encoding
- Integers 10000 to C-1 map to distinct good values (unrank_good is strictly monotonic)
- Integers ≥ C map to distinct base32 strings (base32 is injective)
- The three ranges produce disjoint string sets:
  - Decimal: 1-4 chars, all digits, value ≤ 9999
  - Transition zone: 4 chars, contains at least one letter, base32 value < C
  - Identity zone: 4+ chars, contains at least one letter, base32 value ≥ C

*Surjectivity* (every valid string decodes to some integer):
- All-digit strings ≤ 4 chars decode as decimal
- Strings with letters decode as base32, then:
  - If base32 value v ≥ C: return v
  - If base32 value v < C: return 10000 + rank_good(v)

∎

## Theorem 2: Unambiguity

**Claim**: No valid B1032 string can be interpreted as both decimal and base32.

**Proof**: The decoder uses a simple rule:

```
if (token is all-digits AND length ≤ 4):
    decode as decimal
else:
    decode as base32
```

This is unambiguous because:
1. All-digit tokens ≤ 4 chars are ALWAYS decimal (by definition)
2. The encoder NEVER produces all-digit tokens ≤ 4 chars for values ≥ 10000
   - Values 10000 to C-1 map to good values (contain letters)
   - Values ≥ C map to base32 values ≥ C (which contain letters by Lemma 2)

Therefore the encoding/decoding rules partition the space completely. ∎

## Theorem 3: Roundtrip Correctness

**Claim**: For all n ∈ ℕ: `decode(encode(n)) = n`

**Proof by cases**:

**Case 1: n ≤ 9999**
- encode(n) = decimal string of n
- decode sees all-digits, length ≤ 4 → parses as decimal → returns n ✓

**Case 2: 10000 ≤ n < C**
- encode(n) = to_base32_min4(unrank_good(n - 10000))
- Let v = unrank_good(n - 10000). By construction, v is a good value.
- decode sees a string with letters → parses as base32 → gets v
- Since v < C, decode returns 10000 + rank_good(v)
- Since rank_good(unrank_good(k)) = k for all k, we get 10000 + (n - 10000) = n ✓

**Case 3: n ≥ C**
- encode(n) = to_base32_min4(n) (identity)
- decode sees a string with letters → parses as base32 → gets n
- Since n ≥ C, decode returns n directly ✓

∎

## Theorem 4: Lexicographic Order Preservation

**Claim**: For most practical ranges, n₁ < n₂ implies encode(n₁) < encode(n₂) lexicographically.

**Proof sketch**:
- Within decimal zone [0, 9999]: Decimal strings of equal length are lexicographically ordered
- Within transition zone [10000, C): good values are enumerated in order
- Within identity zone [C, B): base32 is lexicographically ordered
- Cross-zone: 4-char tokens starting with '0' come before those starting with '9'

The ordering isn't perfect across all boundaries (e.g., "9999" > "000A" lexicographically) but is preserved within each zone and across the transition/identity boundary. ∎

## Implementation Correctness

The key functions are:

### `bad_leq(v)`: Count of bad values in [0, v]

Uses digit DP to count base32 values ≤ v where all 4 digits are < 10.

**Invariant**: After processing digit i, `tight` = count of prefixes exactly matching v's prefix, `loose` = count of prefixes strictly less than v's prefix, both restricted to all-digit values.

### `good_leq(v)` = (v + 1) - bad_leq(v)

Correct by definition: total values minus bad values = good values.

### `rank_good(v)` = good_leq(v) - 1

0-indexed rank among good values.

### `unrank_good(k)`: Find k-th good value

Binary search for smallest v where good_leq(v) = k + 1. Correct because good_leq is monotonically increasing.

## Conclusion

B1032 is a **provably correct** bijective encoding with the following guarantees:
1. Every non-negative integer encodes to exactly one string
2. Every valid string decodes to exactly one integer
3. Roundtrip is the identity function
4. No ambiguity between decimal and base32 interpretations

The encoding achieves its design goal: humans see familiar decimal for common values while maintaining density for large values.
