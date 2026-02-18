-- Z855.lean
-- Formal analysis of the z855 encoding invariants.
--
-- The central question: when does the "before block" reconstruction
-- have a unique solution, and when can it fail?
--
-- z855 encodes 4 bytes as 5 base-85 characters.
-- When a passthrough escape appears at position P within a block:
--   - 4-byte escape (`,`): P digits known, (4-P) low bytes known
--   - 5/6/7-byte escape (`;`/`_`/`~`): (P+1) digits known, (4-P) low bytes known
--
-- Before-block reconstruction solves:
--   Find n ∈ [base·85^(4-P), (base+1)·85^(4-P))  s.t.  n ≡ k (mod 256^(4-P))
--
-- Key question: is this solution unique?

-- ============================================================
-- Core uniqueness lemma
-- ============================================================

-- A range of size < modulus contains at most one element of any congruence class.
-- Proof: two solutions differ by < size < modulus; equal residues force difference = 0.
theorem unique_in_small_range
    (lo size modulus : Nat)
    (hsize : size < modulus)
    (n m   : Nat)
    (hn    : lo ≤ n ∧ n < lo + size)
    (hm    : lo ≤ m ∧ m < lo + size)
    (hcong : n % modulus = m % modulus) : n = m := by
  rcases Nat.lt_or_ge n m with hlt | hge
  · -- n < m: show modulus ∣ (m-n), and m-n < modulus, so m-n = 0.
    have hn_eq := Nat.div_add_mod n modulus  -- modulus * (n/modulus) + n%modulus = n
    have hm_eq := Nat.div_add_mod m modulus
    have hq : n / modulus ≤ m / modulus := Nat.div_le_div_right (Nat.le_of_lt hlt)
    have hdvd : modulus ∣ (m - n) :=
      ⟨m / modulus - n / modulus, by rw [Nat.mul_sub]; omega⟩
    have hbnd : m - n < modulus := by omega
    have := Nat.eq_zero_of_dvd_of_lt hdvd hbnd
    omega
  · -- n ≥ m: symmetric.
    have hn_eq := Nat.div_add_mod n modulus
    have hm_eq := Nat.div_add_mod m modulus
    have hq : m / modulus ≤ n / modulus := Nat.div_le_div_right hge
    have hdvd : modulus ∣ (n - m) :=
      ⟨n / modulus - m / modulus, by rw [Nat.mul_sub]; omega⟩
    have hbnd : n - m < modulus := by omega
    have := Nat.eq_zero_of_dvd_of_lt hdvd hbnd
    omega

-- ============================================================
-- The key numerical inequalities
-- ============================================================

-- Extended passthrough: range 85^(4-P) vs modulus 256^(4-P).
-- Since 85 < 256: range < modulus → unique solution.
example : 85^4 < 256^4 := by native_decide   -- P=0: 52200625 < 4294967296 ✓
example : 85^3 < 256^3 := by native_decide   -- P=1: 614125   < 16777216   ✓
example : 85^2 < 256^2 := by native_decide   -- P=2: 7225     < 65536      ✓
example : 85^1 < 256^1 := by native_decide   -- P=3: 85       < 256        ✓

-- 4-byte passthrough: range 85^(5-P) vs modulus 256^(4-P).
-- Range EXCEEDS modulus → multiple solutions possible.
example : 85^4 > 256^3 := by native_decide   -- P=1: 52200625 > 16777216 ✗
example : 85^3 > 256^2 := by native_decide   -- P=2: 614125   > 65536    ✗
example : 85^2 > 256^1 := by native_decide   -- P=3: 7225     > 256      ✗

-- Max solutions for 4-byte passthrough (ceiling division):
#eval (85^4 + 256^3 - 1) / 256^3   -- P=1: up to 4
#eval (85^3 + 256^2 - 1) / 256^2   -- P=2: up to 10
#eval (85^2 + 256^1 - 1) / 256^1   -- P=3: up to 29

-- ============================================================
-- Main theorems: uniqueness for each P case
-- ============================================================

-- P=0: 1 digit, 4 known bytes. Range 85^4=52200625, mod 256^4=4294967296.
theorem extended_unique_P0 (base k n m : Nat)
    (hn : base * 52200625 ≤ n ∧ n < (base+1) * 52200625)
    (hm : base * 52200625 ≤ m ∧ m < (base+1) * 52200625)
    (hn_c : n % 4294967296 = k) (hm_c : m % 4294967296 = k) : n = m :=
  unique_in_small_range (base * 52200625) 52200625 4294967296 (by native_decide) n m
    ⟨hn.1, by omega⟩ ⟨hm.1, by omega⟩ (by rw [hn_c, hm_c])

-- P=1: 2 digits, 3 known bytes. Range 85^3=614125, mod 256^3=16777216.
theorem extended_unique_P1 (base k n m : Nat)
    (hn : base * 614125 ≤ n ∧ n < (base+1) * 614125)
    (hm : base * 614125 ≤ m ∧ m < (base+1) * 614125)
    (hn_c : n % 16777216 = k) (hm_c : m % 16777216 = k) : n = m :=
  unique_in_small_range (base * 614125) 614125 16777216 (by native_decide) n m
    ⟨hn.1, by omega⟩ ⟨hm.1, by omega⟩ (by rw [hn_c, hm_c])

-- P=2: 3 digits, 2 known bytes. Range 85^2=7225, mod 256^2=65536.
theorem extended_unique_P2 (base k n m : Nat)
    (hn : base * 7225 ≤ n ∧ n < (base+1) * 7225)
    (hm : base * 7225 ≤ m ∧ m < (base+1) * 7225)
    (hn_c : n % 65536 = k) (hm_c : m % 65536 = k) : n = m :=
  unique_in_small_range (base * 7225) 7225 65536 (by native_decide) n m
    ⟨hn.1, by omega⟩ ⟨hm.1, by omega⟩ (by rw [hn_c, hm_c])

-- P=3: 4 digits, 1 known byte. Range 85^1=85, mod 256^1=256.
theorem extended_unique_P3 (base k n m : Nat)
    (hn : base * 85 ≤ n ∧ n < (base+1) * 85)
    (hm : base * 85 ≤ m ∧ m < (base+1) * 85)
    (hn_c : n % 256 = k) (hm_c : m % 256 = k) : n = m :=
  unique_in_small_range (base * 85) 85 256 (by native_decide) n m
    ⟨hn.1, by omega⟩ ⟨hm.1, by omega⟩ (by rw [hn_c, hm_c])

-- ============================================================
-- Counter-example: 4-byte passthrough ambiguity is real
-- ============================================================

-- At P=3, range [0, 7225), mod 256: 29 distinct values share residue 42.
example : ((List.range 7225).filter (fun n => n % 256 = 42)).length = 29 := by
  native_decide

-- ============================================================
-- Summary
-- ============================================================
--
-- PROVED: 5/6/7-byte extended passthrough uniquely reconstructs the before-block.
--
--   Core lemma (unique_in_small_range):
--     If size < modulus, any congruence class n ≡ k (mod modulus) contains
--     at most one element in any range [lo, lo+size).
--     Proof: two solutions differ by < size < modulus; equal residues force
--     modulus | difference, but difference < modulus → difference = 0.
--
--   Applied to z855 (all four P cases proved above):
--     P=0: 85^4 = 52200625  < 4294967296 = 256^4  ✓
--     P=1: 85^3 = 614125    < 16777216   = 256^3  ✓
--     P=2: 85^2 = 7225      < 65536      = 256^2  ✓
--     P=3: 85^1 = 85        < 256        = 256^1  ✓
--
-- PROVED by counter-example: 4-byte passthrough can have many solutions.
--   P=1: up to 4;  P=2: up to 10;  P=3: up to 29.
--   Canonical minimum is necessary, not defensive.
--
-- ROOT CAUSE of the asymmetry:
--   Extended passthrough outputs P+1 digits instead of P.
--   This shrinks the range by a factor of 85 (from 85^(5-P) to 85^(4-P)).
--   Since 85 < 256, we always get 85^(4-P) < 256^(4-P).
--   One extra Z85 digit is exactly sufficient — and it works because
--   the base-85 alphabet is coarser than a byte (85 < 256).
