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
      simp [Nat.pow_succ, Nat.mul_comm]
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

-- ============================================================
-- Computational convergence certificates
-- ============================================================

-- A cleaner convergence check: does n reach 1 within `fuel` steps?
def convergesIn : Nat → Nat → Bool
  | _,      1 => true
  | 0,      _ => false
  | f + 1,  n => convergesIn f (collatz n)

-- Sanity checks
#eval convergesIn 200 27    -- true (takes 111 steps)
#eval convergesIn 50  27    -- false (not enough fuel)

-- This gives us a *certificate*: to prove collatzConverges n,
-- just exhibit a fuel value and show convergesIn fuel n = true.
-- 0 is a fixed point of collatz (0 % 2 = 0, 0 / 2 = 0)
theorem collatz_zero : collatz 0 = 0 := by native_decide

-- Therefore 0 never converges (convergesIn always returns false for 0)
theorem convergesIn_zero_false : ∀ fuel, convergesIn fuel 0 = false := by
  intro fuel
  induction fuel with
  | zero => rfl
  | succ f ih =>
    show convergesIn f (collatz 0) = false
    rw [collatz_zero]; exact ih

theorem convergesIn_correct {fuel n : Nat} (h : convergesIn fuel n = true) :
    collatzConverges n := by
  induction fuel generalizing n with
  | zero =>
    -- convergesIn 0 n: only | _, 1 => true fires; everything else is false
    match n with
    | 0     =>
      -- convergesIn 0 0 = false (| 0, _ => false pattern)
      exact absurd h (convergesIn_zero_false 0)
    | 1     => exact ⟨0, rfl⟩
    | n + 2 =>
      -- convergesIn 0 (n+2) = false definitionally (n+2 ≠ 1, fuel=0)
      have hf : convergesIn 0 (n + 2) = false := rfl
      rw [hf] at h; simp at h
  | succ f ih =>
    match n with
    | 0     => exact absurd h (convergesIn_zero_false (f + 1))
    | 1     => exact ⟨0, rfl⟩
    | n + 2 =>
      -- convergesIn (f+1) (n+2) reduces to convergesIn f (collatz (n+2)) definitionally
      have h' : convergesIn f (collatz (n + 2)) = true := h
      obtain ⟨k, hk⟩ := ih h'
      exact ⟨k + 1, by simp [collatzN, hk]⟩

-- Computationally verify: all n ∈ [1..100] converge (within 10000 steps)
-- native_decide computes this efficiently at compile time
theorem all_converge_below_100 :
    ∀ n : Fin 101, 0 < n.val → convergesIn 10000 n.val = true := by
  native_decide

-- Corollary: all n ≤ 100 satisfy the Collatz conjecture
theorem collatz_verified_100 :
    ∀ n : Nat, 0 < n → n ≤ 100 → collatzConverges n := by
  intro n hpos hle
  apply convergesIn_correct
  have h := all_converge_below_100 ⟨n, by omega⟩ hpos
  exact h

-- ============================================================
-- The only cycle up to 100 is {1, 2, 4}
-- ============================================================

-- A number n > 0 is "in the canonical cycle" if its trajectory from 1
-- passes through n. The only such numbers ≤ 100 are 1, 2, 4.

-- More practically: no n ≤ 100 (other than 1, 2, 4) satisfies collatzN k n = n
-- for any k ≤ 20. We check this by deciding the bounded version.
def isPeriodicIn (maxK n : Nat) : Bool :=
  (List.range maxK).any (fun k => collatzN (k + 1) n == n)

-- 1, 2, 4 are all periodic (part of the known cycle):
#eval isPeriodicIn 10 1   -- true (period 3: 1→4→2→1)
#eval isPeriodicIn 10 2   -- true
#eval isPeriodicIn 10 4   -- true
#eval isPeriodicIn 10 27  -- false (converges to 1, doesn't cycle back)

-- No n ∈ [5..100] is periodic within 500 steps:
example : ∀ n : Fin 101, 4 < n.val →
    isPeriodicIn 500 n.val = false := by native_decide

-- ============================================================
-- Connection to Busy Beaver and Antihydra
-- ============================================================

-- BB(5) = 47,176,870 was proved in 2024 by the bbchallenge community,
-- formally verified in Rocq (Coq) by the mysterious contributor "mxdys".
-- The proof ran automated decision procedures on all 5-state Turing machines,
-- leaving 13 "sporadic machines" requiring individual non-halting proofs.
-- One machine was the champion (halts after exactly 47,176,870 steps).
-- Reference: https://github.com/ccz181078/Coq-BB5

-- For BB(6), the barrier is "Antihydra" — a 6-state 2-symbol TM whose
-- behavior is structurally similar to the Collatz conjecture.
-- Determining whether Antihydra halts would require solving a Collatz-like
-- open problem, which is why BB(6) may be unknowable with current mathematics.

-- This connects Collatz to the frontier of formal computability theory:
-- the reason BB(n) gets hard is precisely that longer-running machines
-- can encode open conjectures like Collatz as their halting condition.

-- The shortcut Collatz map is essentially a 2-state, infinite-tape machine
-- with a simple update rule — exactly the kind of thing busy beaver hunters
-- fear: simple rule, complex behavior, no known invariant.

-- Our shortcutCollatz theorem above already shows the structure:
-- odd → even is forced (odd_step_produces_even), giving the "2-step" view.
-- But the interleaving of growth (×1.5) and shrinkage (÷2) has no
-- algebraic invariant we can exploit for a general termination proof.
