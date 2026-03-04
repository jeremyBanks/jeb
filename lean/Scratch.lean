-- Scratch.lean
-- Formalizing the HALT bug state machine and z855 encoding properties.

-- ============================================================
-- Part 1: HALT case analysis
-- ============================================================

inductive HaltCase where
  | case1 : HaltCase  -- IME=1, pending=false → wait for interrupt
  | case2 : HaltCase  -- IME=1, pending=true  → exits (dispatched next tick)
  | case3 : HaltCase  -- IME=0, pending=false → wait, exit clean (no bug)
  | case4 : HaltCase  -- IME=0, pending=true  → halt bug: PC byte duplicated

def classifyHalt (ime pending : Bool) : HaltCase :=
  match ime, pending with
  | true,  false => .case1
  | true,  true  => .case2
  | false, false => .case3
  | false, true  => .case4

def isHaltBug : HaltCase → Bool
  | .case4 => true
  | _      => false

theorem halt_bug_iff (ime pending : Bool) :
    isHaltBug (classifyHalt ime pending) = true ↔ (ime = false ∧ pending = true) := by
  cases ime <;> cases pending <;> simp [classifyHalt, isHaltBug]

-- The `halting` flag records "HALT was entered with pending=false" (Case 3).
-- When re-executing and pending becomes true: halting=true → clean exit, no bug.
def shouldSetHaltBug (haltingAtEntry ime pending : Bool) : Bool :=
  if haltingAtEntry then false else isHaltBug (classifyHalt ime pending)

theorem halt_bug_logic :
    shouldSetHaltBug true  false true  = false ∧  -- Case 3 exit: no bug
    shouldSetHaltBug false false true  = true  ∧  -- Case 4: bug fires
    shouldSetHaltBug false true  true  = false ∧  -- IME=1: no bug
    shouldSetHaltBug false true  false = false :=
  by simp [shouldSetHaltBug, isHaltBug, classifyHalt]

-- halting=true is a global invariant: always prevents bug regardless of other state
theorem halting_prevents_bug (ime pending : Bool) :
    shouldSetHaltBug true ime pending = false := by
  simp [shouldSetHaltBug]

-- Full characterization
theorem bug_iff_not_halting_and_case4 (halting ime pending : Bool) :
    shouldSetHaltBug halting ime pending = true ↔
    (halting = false ∧ ime = false ∧ pending = true) := by
  cases halting <;> cases ime <;> cases pending <;>
    simp [shouldSetHaltBug, isHaltBug, classifyHalt]

-- ============================================================
-- Part 2: z855 encoding
-- ============================================================

-- 85^5 > 256^4: five base-85 digits can represent any 4-byte value
theorem base85_headroom : 85^5 > 256^4 := by native_decide

#eval 85^5 - 256^4  -- 142,085,829 unused 5-tuples out of 85^5

-- Helper
theorem fifth_digit_bound {n : Nat} (hn : n < 256^4) : n / 85^4 < 85 := by
  have h1 : (256:Nat)^4 = 4294967296 := by native_decide
  have h2 : (85:Nat)^4   = 52200625  := by native_decide
  omega

-- z855 encoding as a plain function (avoiding Fin complications)
-- Input:  n in [0, 256^4)
-- Output: (d0, d1, d2, d3, d4) where each di in [0, 85)
--   d0 = n % 85
--   d1 = (n / 85) % 85
--   d2 = (n / 85^2) % 85
--   d3 = (n / 85^3) % 85
--   d4 = n / 85^4          (guaranteed < 85 by fifth_digit_bound)

-- Round-trip: the polynomial reconstruction recovers n exactly
-- The algebraic identity holds for any n, not just those in range
theorem encode_decode_roundtrip (n : Nat) :
    let d0 := n % 85
    let d1 := (n / 85) % 85
    let d2 := (n / 85^2) % 85
    let d3 := (n / 85^3) % 85
    let d4 := n / 85^4
    d0 + 85 * d1 + 85^2 * d2 + 85^3 * d3 + 85^4 * d4 = n := by
  simp only []
  omega

-- Injectivity: equal encodings imply equal inputs
theorem encodeValue_injective (a b : Nat) (ha : a < 256^4) (hb : b < 256^4)
    (h0 : a % 85 = b % 85)
    (h1 : (a / 85) % 85 = (b / 85) % 85)
    (h2 : (a / 85^2) % 85 = (b / 85^2) % 85)
    (h3 : (a / 85^3) % 85 = (b / 85^3) % 85)
    (h4 : a / 85^4 = b / 85^4) : a = b := by
  omega

-- ============================================================
-- Part 3: Timer coverage — the math behind the LDH bug fix
-- ============================================================
-- sync_tima polls TIMA every P M-cycles; timer ticks every 16 M-cycles.
-- Loop hits all 16 phase windows iff gcd(P, 16) = 1 (i.e., P is odd).
--
-- Bug:  P=12, gcd(12,16)=4 → only {0,4,8,12} reachable → stuck forever
-- Fix:  P=11, gcd(11,16)=1 → all 16 residues reachable → loop exits

example : Nat.gcd 12 16 = 4 := by native_decide  -- buggy period: stuck
example : Nat.gcd 11 16 = 1 := by native_decide  -- fixed period: all phases

-- Orbit sizes (count of distinct residues):
#eval ((List.range 16).map (fun k => k * 12 % 16)).eraseDups.length  -- 4
#eval ((List.range 16).map (fun k => k * 11 % 16)).eraseDups.length  -- 16

example : ((List.range 16).map (fun k => k * 12 % 16)).eraseDups.length = 4  := by native_decide
example : ((List.range 16).map (fun k => k * 11 % 16)).eraseDups.length = 16 := by native_decide

-- The formula orbit_size = N / gcd(P, N):
example : 16 / Nat.gcd 12 16 = 4  := by native_decide
example : 16 / Nat.gcd 11 16 = 16 := by native_decide

-- Main theorem: with the corrected period (11), every phase mod 16 is reachable.
-- This formally justifies why the LDH timing fix unlocks the sync loop.
theorem all_phases_reachable_after_fix :
    ∀ target : Fin 16, ∃ k : Fin 16, k.val * 11 % 16 = target.val := by
  native_decide
