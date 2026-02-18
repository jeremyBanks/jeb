-- Collatz.lean
-- Exploring the Collatz conjecture in Lean 4.
-- Goal: not to prove it, but to formalize what it *says* precisely,
-- and prove some interesting things around it.

-- ============================================================
-- The Collatz function
-- ============================================================

-- The Collatz step: n → n/2 if n even, 3n+1 if n odd
-- (We use the "shortcut" form: 3n+1 is always even, so we can divide immediately)
def collatz (n : Nat) : Nat :=
  if n % 2 = 0 then n / 2 else 3 * n + 1

-- The standard iterated map
def collatzN : Nat → Nat → Nat
  | 0,     n => n
  | k + 1, n => collatzN k (collatz n)

-- Some trajectories to feel the shape:
#eval (List.range 20).map (collatzN · 27)
-- 27's famous long journey before descending

-- How many steps to reach 1?
def collatzSteps : Nat → Nat → Nat
  | 0,     _ => 0  -- give up after fuel runs out
  | _,     1 => 0  -- reached 1
  | f + 1, n => 1 + collatzSteps f (collatz n)

#eval collatzSteps 1000 27   -- 111 steps
#eval collatzSteps 1000 9780657631  -- a known large one

-- ============================================================
-- Stating the conjecture precisely
-- ============================================================

-- The conjecture: for every positive n, the sequence eventually reaches 1.
-- This requires termination, which is not obvious.

-- One way to state it: there exists k such that collatzN k n = 1
def collatzConverges (n : Nat) : Prop :=
  ∃ k : Nat, collatzN k n = 1

-- The Collatz conjecture:
def CollatzConjecture : Prop :=
  ∀ n : Nat, 0 < n → collatzConverges n

-- ============================================================
-- Things we CAN prove (no conjecture needed)
-- ============================================================

-- 1. Fixed point: 1 is a fixed point of collatz
theorem collatz_one : collatz 1 = 4 := by native_decide
-- Wait, that's the 3n+1 step: 3*1+1=4, not 1.
-- The *cycle* is 4 → 2 → 1 → 4 → ...
-- So 1 is not a fixed point, but 2 maps to 1 which maps to 4.

-- The trivial cycle: 1 → 4 → 2 → 1
theorem collatz_cycle :
    collatz 1 = 4 ∧ collatz 4 = 2 ∧ collatz 2 = 1 := by native_decide

-- So collatzN 3 1 = 1 (cycle length 3)
theorem one_converges_in_3 : collatzN 3 1 = 1 := by native_decide

-- 2. Powers of 2 converge immediately (they just halve each step)
theorem pow2_converges (k : Nat) : collatzN k (2^k) = 1 := by
  induction k with
  | zero => simp [collatzN]
  | succ k ih =>
    simp only [collatzN, collatz]
    have heven : 2^(k+1) % 2 = 0 := by
      simp [Nat.pow_succ, Nat.mul_comm]
    rw [if_pos heven]
    have hdiv : 2^(k+1) / 2 = 2^k := by
      simp [Nat.pow_succ, Nat.mul_comm, Nat.mul_div_cancel]
    rw [hdiv]
    exact ih

-- 3. Even numbers converge iff their half converges
theorem even_converges_iff (n : Nat) (hn : n % 2 = 0) :
    collatzConverges n ↔ collatzConverges (n / 2) := by
  constructor
  · intro ⟨k, hk⟩
    cases k with
    | zero =>
      simp [collatzN] at hk
      -- n = 1, but n is even, so n = 0 or n ≥ 2. If n=1, contradiction.
      omega
    | succ k =>
      refine ⟨k, ?_⟩
      simp [collatzN, collatz, hn] at hk
      exact hk
  · intro ⟨k, hk⟩
    refine ⟨k + 1, ?_⟩
    simp [collatzN, collatz, hn, hk]

-- 4. If n converges and collatz n = m, then m converges
theorem collatz_converges_step {n : Nat} (h : collatzConverges n) :
    collatzConverges (collatz n) := by
  obtain ⟨k, hk⟩ := h
  cases k with
  | zero =>
    simp [collatzN] at hk
    -- n = 1, collatz 1 = 4, and 4 → 2 → 1 in 2 more steps
    subst hk
    exact ⟨2, by native_decide⟩
  | succ k =>
    exact ⟨k, by simp [collatzN] at hk; exact hk⟩

-- ============================================================
-- Verified cases of the conjecture (by computation)
-- ============================================================

-- Computationally verify small cases (native_decide handles this efficiently):
example : collatzConverges 27 := ⟨111, by native_decide⟩
example : collatzConverges 871 := ⟨178, by native_decide⟩

-- (This would be slow for large bounds, but proves the principle)

-- ============================================================
-- What makes this hard to prove in general?
-- ============================================================

-- The issue: we don't have a decreasing measure.
-- For even n: n/2 < n ✓ (decreasing)
-- For odd n: 3n+1 > n ✗ (increasing!)
--
-- So a naive well-founded recursion doesn't work.
-- The conjecture says the increases are always "temporary" —
-- but formalizing "temporary" requires knowing the sequence terminates,
-- which is exactly what we're trying to prove.

-- This is why the conjecture is hard: no known invariant decreases
-- monotonically under the map. It seems to require global knowledge
-- about the entire trajectory, not just local steps.

-- ============================================================
-- A related thing that IS provable: density of even steps
-- ============================================================

-- Any odd n immediately produces an even number (3n+1 is even when n is odd)
theorem odd_step_produces_even (n : Nat) (hn : n % 2 = 1) :
    collatz n % 2 = 0 := by
  simp [collatz, hn]
  omega

-- So the sequence alternates at worst: odd → even → ??? → ...
-- In fact odd steps always come in pairs with even steps immediately after.
-- This means on average, the sequence decreases by factor ~3/4 per two steps
-- (3n+1 then /2 = (3n+1)/2 ≈ 1.5n, but then likely even again → /2 ≈ 0.75n)

-- The "shortcut" version of Collatz that combines these:
def collatzShortcut (n : Nat) : Nat :=
  if n % 2 = 0 then n / 2 else (3 * n + 1) / 2

-- For odd n, the shortcut gives the result after two standard steps
theorem shortcut_odd (n : Nat) (hn : n % 2 = 1) :
    collatzShortcut n = collatz (collatz n) := by
  unfold collatz collatzShortcut
  have heven : (3 * n + 1) % 2 = 0 := by omega
  simp [hn, heven]

-- ============================================================
-- Connection to our timer orbit theorem
-- ============================================================

-- Yesterday we proved: if gcd(P, N) = 1, the orbit of 0 under "add P mod N"
-- covers all of ℤ/Nℤ. This is the finite, decidable version of orbit coverage.
--
-- Collatz is asking about orbits of a map on ℕ — infinite, not decidable,
-- and the map is not a group action. So our timer proof technique doesn't
-- transfer. The difficulty is exactly the loss of algebraic structure.
--
-- The timer worked because the orbit was on a *finite* cyclic group where
-- gcd gives us a complete invariant. Collatz has no known analogous invariant.

-- ============================================================
-- Fun: what's the longest trajectory under 1000?
-- ============================================================

-- 871 is the champion under 1000 (178 steps):
#eval collatzSteps 500 871   -- 178
