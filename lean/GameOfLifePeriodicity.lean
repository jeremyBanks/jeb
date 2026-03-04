-- GameOfLifePeriodicity.lean
-- Exploring periodicity in Conway's Game of Life on finite toroidal grids.
--
-- We focus on the 8×1 toroidal grid: 8 cells in a ring.
-- This is small enough to be fully enumerable (2^8 = 256 states)
-- yet rich enough to have interesting period structure.
--
-- The 1D ring Life rule: a cell is born/survives based on its
-- two neighbours (and itself) — but we use the standard 2D rule
-- restricted to a degenerate grid where height=1, so each cell
-- has exactly 5 neighbours: left, right, upper-left, upper-right, up.
-- On a 1×W torus, "up" and "down" wrap to the same row,
-- so each cell sees: left, right, and itself × 3 (above, below, and itself).
-- That gives: count = 2*(left + right) + self, with birth at count=3, survival at 2 or 3.
-- Actually let's use the clean 1D interpretation: 3-cell neighbourhood, standard outer-totalistic.
--
-- For the 8×1 torus with the *standard 2D rule* collapsed to 1D:
-- In a 1-row torus of width W, each cell (0,0) has neighbours:
-- (-1,-1) (-1,0) (-1,1) (0,-1) (0,1) (1,-1) (1,0) (1,1)
-- but all y coordinates are mod 1 = 0, so:
-- (-1,0) appears 3 times (y=-1,0,1 all = 0 mod 1) — wait, that's wrong.
-- On a true 1-row torus y is fixed at 0, so y±1 wraps to 0.
-- Neighbours of (x,0): (x-1,0),(x+1,0),(x-1,0),(x+1,0),(x-1,0),(x,0),(x+1,0),(x,0)
-- = 3*(x-1,0) + 3*(x+1,0) + 2*(x,0)  ← but (x,0) is the cell itself, not counted.
-- The 8 offsets give: left×3, right×3, self×2 — but self isn't counted as neighbour.
-- So live_neighbours = 3*left + 3*right (self-overlap slots don't count).
-- Birth: 3*left+3*right = 3 → left+right = 1 (exactly one of the two neighbours alive)
-- Survival: 3*left+3*right ∈ {2,3} → impossible since it's a multiple of 3.
--   Actually 3*(left+right) ∈ {2,3} → no solution. So NO cells survive! All die or are born.
-- This is degenerate. Let's use the *proper* 1D interpretation instead.
--
-- CLEAN INTERPRETATION: 8-cell ring with standard Life but each cell only has
-- 2 neighbours (left and right on the ring). Rule:
--   alive, 1 neighbour → dies
--   alive, 2 neighbours → survives
--   dead,  2 neighbours → born... no wait, that's not standard.
-- Let's just define a clean "1D outer-totalistic" rule matching Life's flavour:
--
-- RULE B3/S23 on the ring (treating the ring as 1D, neighbourhood = {left, right, self}):
-- Count = left + right  (0, 1, or 2)
-- Birth:   count = 1   (if dead and exactly 1 neighbour alive)
-- Survive: count = 1   (if alive and exactly 1 neighbour alive)
-- This is actually rule 110-ish. Let's just use Conway's exact B3/S23:
--   Birth:   count = 3... but max count is 2 on a 1D ring. No births. Boring.
--
-- BEST INTERPRETATION: use the FULL 2D rule but on a W×2 torus (two rows),
-- or more naturally: enumerate all 2^8=256 states of an 8-cell ring and
-- define the evolution function, then find all periods by computation.
--
-- We'll use a pure computational approach: represent state as Fin 256 (= BitVec 8)
-- and define the step function, then prove period bounds by exhaustive search.

-- ============================================================
-- 8-cell ring with B3/S23-like rule
-- We use the 1D "Life" rule with neighbourhood size 3:
-- Count = left + center + right
-- Born if count = 2 (using center as "self" doesn't affect birth)
-- Actually let's just use the ECA (elementary cellular automaton) framework
-- and define a specific rule, then study its period structure.
-- ============================================================

-- A state of the 8-cell ring as a function Fin 8 → Bool
def RingState := Fin 8 → Bool

-- Wrap index mod 8
def wrap (i : Int) : Fin 8 := ⟨(i % 8).toNat % 8, by omega⟩

-- Neighbourhood count: left + right neighbours on the ring
-- (not including self, matching Life's neighbour-counting convention)
def ringNeighbours (s : RingState) (i : Fin 8) : Nat :=
  (if s ⟨(i.val + 7) % 8, by omega⟩ then 1 else 0) +
  (if s ⟨(i.val + 1) % 8, by omega⟩ then 1 else 0)

-- 1D Life rule: B1/S12 (interesting for rings)
-- Born if exactly 1 live neighbour, survive if 1 or 2 live neighbours
-- (the "simplest" non-trivial 1D rule with both birth and survival)
def ringStep1D (s : RingState) : RingState := fun i =>
  let n := ringNeighbours s i
  if s i then n == 1 || n == 2  -- survive if 1 or 2 neighbours
  else n == 1                    -- born if exactly 1 neighbour

-- Pack/unpack RingState ↔ Nat for efficient computation
def stateToN (s : RingState) : Nat :=
  (if s ⟨0, by omega⟩ then 1   else 0) |||
  (if s ⟨1, by omega⟩ then 2   else 0) |||
  (if s ⟨2, by omega⟩ then 4   else 0) |||
  (if s ⟨3, by omega⟩ then 8   else 0) |||
  (if s ⟨4, by omega⟩ then 16  else 0) |||
  (if s ⟨5, by omega⟩ then 32  else 0) |||
  (if s ⟨6, by omega⟩ then 64  else 0) |||
  (if s ⟨7, by omega⟩ then 128 else 0)

def nToState (n : Nat) : RingState :=
  fun i => (n >>> i.val) &&& 1 == 1

-- Step function on Nat representation
def stepN (n : Nat) : Nat :=
  stateToN (ringStep1D (nToState n))

-- ============================================================
-- Orbit analysis: compute the orbit of a state under iteration
-- ============================================================

-- Iterate the step function k times
def iterN (k : Nat) (n : Nat) : Nat :=
  match k with
  | 0 => n
  | k+1 => iterN k (stepN n)

-- The orbit of a state (first K steps)
def orbit (n : Nat) (k : Nat) : List Nat :=
  (List.range k).map (fun i => iterN i n)

-- Verify by computation:
#eval! orbit 0b00001110 20  -- glider-like pattern?
#eval! orbit 0b11111111 10  -- all alive
#eval! orbit 0b10101010 10  -- alternating
#eval! orbit 0b00010000 20  -- single cell

-- ============================================================
-- Period detection: find the period of a state
-- ============================================================

def findPeriod (n : Nat) (maxK : Nat) : Option Nat :=
  let start := n
  let rec search : Nat → Nat → Option Nat
    | 0,   _   => none
    | k+1, cur =>
      let next := stepN cur
      if next == start then some 1
      else match search k next with
        | none => none
        | some p => some (p + 1)
  search maxK (stepN start)

-- All periods in the 8-cell ring (exhaustive over all 256 states)
def allPeriods : List (Nat × Option Nat) :=
  (List.range 256).map (fun n => (n, findPeriod n 300))

-- States that reach a fixed point (period 1 = still life)
def stillLifeStates : List Nat :=
  (List.range 256).filter (fun n => stepN n == n)

-- States with period 2
def period2States : List Nat :=
  (List.range 256).filter (fun n =>
    stepN n != n && iterN 2 n == n)

-- (using #eval! since these depend on sorry-free but opaque definitions)
#eval stillLifeStates
#eval period2States
#eval stillLifeStates.length
#eval period2States.length

-- Maximum period across all 256 states
#eval (List.range 256).foldl (fun maxP n =>
  match findPeriod n 300 with
  | none => maxP
  | some p => max maxP p) 0

-- ============================================================
-- Key theorem: the 8-cell ring eventually cycles
-- Proof: state space is finite (256 states), so by pigeonhole
-- any orbit must revisit a state within 256 steps.
-- ============================================================

-- The step function is a function Fin 256 → Fin 256
-- so the orbit is eventually periodic (pigeonhole on finite set)

-- stepN maps [0,255] to [0,255] (the state space is closed)
theorem stepN_bounded : ∀ n : Nat, n < 256 → stepN n < 256 := by native_decide

-- Every state in [0,255] eventually returns to itself.
-- We verify this computationally: for each of the 256 states,
-- check that iterN p n = n for some p ≤ 256.
-- (native_decide can't synthesize Decidable for ∃ over Nat, so we use a helper)

-- All orbits eventually repeat: pigeonhole on 256 states.
-- An orbit of length 257 on a 256-state space must have a repeat.
def orbitList (n : Nat) (k : Nat) : List Nat :=
  (List.range k).map (fun i => iterN i n)

def hasRepeat (n : Nat) (k : Nat) : Bool :=
  let o := orbitList n k
  o.length != o.eraseDups.length

-- Verify computationally: every starting state has a repeat within 257 steps
#eval (List.range 256).all (fun n => hasRepeat n 257)

-- Proved by native_decide (exhaustive over 256 × 257 = small computation)
theorem ring_eventually_periodic :
    (List.range 256).all (fun n => hasRepeat n 257) = true := by native_decide

-- The attractor: after 256 steps the transient is over and we're on the cycle
def findAttractor (n : Nat) : Nat := iterN 256 n

-- How many distinct attractors are there?
#eval ((List.range 256).map findAttractor).eraseDups.length  -- 22
#eval ((List.range 256).map findAttractor).eraseDups         -- the 22 still lifes!

-- KEY FINDING: all 22 attractors are still lifes (period-1 fixed points)
-- i.e., the 8-cell B1/S12 ring has NO oscillators — every state settles.

-- Verify: every attractor is a fixed point of stepN
#eval ((List.range 256).map findAttractor).eraseDups.all (fun n => stepN n == n)

-- Prove it: every state converges to a still life within 256 steps
theorem ring_converges_to_still_life :
    ∀ n : Nat, n < 256 → stepN (findAttractor n) = findAttractor n := by
  native_decide

-- So the 8-cell B1/S12 ring is "non-oscillatory": no period-2 or higher cycles.
-- Every trajectory: transient → still life. The rule is "boring" in this regard,
-- but the 22 still lifes have rich structure (rotation orbits of size 1 and 8).

-- The 22 still lifes decompose into rotation orbits.
-- A still life with period-1 rotation symmetry (looks same rotated) contributes 1.
-- A still life with trivial rotation symmetry contributes 8.
-- Let's count:
#eval stillLifeStates.filter (fun n => stateToN (rotate (nToState n)) == n)  -- rotationally symmetric
-- The rest come in rotation orbits of size 8 (or 4, 2, 1 dividing 8)

-- Summary:
-- 8-cell ring under B1/S12:
--   256 states total
--   22 still lifes (fixed points)
--   0  oscillators (no periodic orbits with period > 1)
--   All 256 states eventually reach a still life (no limit cycles with period > 1)

-- ============================================================
-- What IS the maximum period on the 8-cell ring?
-- ============================================================

-- Let's compute it and then try to prove it as a bound
#eval ((List.range 256).filterMap (fun n => findPeriod n 300)).foldl max 0

-- Formally state the max-period bound (fill in after computing)
-- theorem ring_max_period : ∀ n : Nat, n < 256 →
--     ∃ p : Nat, p ≤ MAX_PERIOD ∧ iterN p n = n := by
--   native_decide

-- ============================================================
-- Symmetry: the ring has a rotation symmetry
-- Step commutes with rotation by 1
-- ============================================================

def rotate (s : RingState) : RingState :=
  fun i => s ⟨(i.val + 1) % 8, by omega⟩

-- Key: index arithmetic for the rotation commutativity
-- (i+7)%8 + 1)%8 = (i+1+7)%8 and ((i+1)%8+1)%8 = (i+2)%8 = (i+1+1)%8
private theorem rot_left (i : Fin 8) :
    (((i.val + 7) % 8 + 1) % 8) = ((i.val + 1 + 7) % 8) := by
  have h := i.isLt; omega

private theorem rot_right (i : Fin 8) :
    (((i.val + 1) % 8 + 1) % 8) = ((i.val + 1 + 1) % 8) := by
  have h := i.isLt; omega

-- Rotation commutes with step
-- Strategy: show that at each cell i, the step output is the same
-- whether we rotate-then-step or step-then-rotate.
-- The key: after rotation, neighbours of i in (rotate s) are
-- the same cells as neighbours of (i+1) in s.
-- Rotation commutes with step: proved by showing index arithmetic works out.
-- The key identities (for i : Fin 8):
--   ((i+7)%8 + 1) % 8 = (i+1+7) % 8   [left neighbour shifts with rotation]
--   ((i+1)%8 + 1) % 8 = (i+1+1) % 8   [right neighbour shifts with rotation]
-- We prove this cell-by-cell via Fin.val case analysis (8 cases).
-- Rotation commutes with step.
-- We prove this at the Nat/index level: for all i < 8, the step output
-- at position i after rotating equals rotating the step output at i.
-- The key index identities (i < 8):
--   left of i after rotate  = left of rotate(i)  in original
--   right of i after rotate = right of rotate(i) in original
-- Helper: the left-neighbour index commutes with rotation
-- ((i+7)%8+1)%8 = (i+1+7)%8  (both equal (i+8)%8 = i%8 when i<8... wait, not quite)
-- Actually: ((i+7)%8+1)%8 = (i%8) since (i+7)%8 = i-1 mod 8, then +1 = i.
-- And (i+1+7)%8 = (i+8)%8 = i%8. So both = i%8. They're equal!
private theorem left_nb_idx (i : Fin 8) :
    ((i.val + 7) % 8 + 1) % 8 = (i.val + 1 + 7) % 8 := by
  have := i.isLt; omega

theorem step_commutes_with_rotation (s : RingState) :
    ringStep1D (rotate s) = rotate (ringStep1D s) := by
  funext i
  -- Unfold everything and then use the fact that Fin equality is decidable
  -- and the state space is Fin 8 → Bool, so we can use congrArg s (Fin.ext ...)
  simp only [ringStep1D, rotate, ringNeighbours]
  -- The left-neighbour index on LHS is ((i+7)%8+1)%8, on RHS is ((i+1)%8+7)%8.
  -- These are definitionally equal as Nats (by omega) so the Fins are equal.
  -- We use congrArg to rewrite s applied to these Fins.
  -- Two index rewrites needed:
  --   ((i+7)%8+1)%8 = (i+1+7)%8     [left nb of i after rotate = left nb of rotate(i)]
  --   (i+1+7)%8     = ((i+1)%8+7)%8  [same thing written differently on RHS]
  -- Together: replace LHS left-nb index with RHS left-nb index.
  have h1 : ((i.val + 7) % 8 + 1) % 8 = ((i.val + 1) % 8 + 7) % 8 := by
    have := i.isLt; omega
  have heq1 : (⟨((i.val + 7) % 8 + 1) % 8, by omega⟩ : Fin 8) =
              ⟨((i.val + 1) % 8 + 7) % 8, by omega⟩ := Fin.ext h1
  simp only [congrArg s heq1]

-- Key: iteration commutes with rotation (by induction on steps)
theorem iter_commutes_with_rotation (s : RingState) (p : Nat) :
    (ringStep1D^[p]) (rotate s) = rotate ((ringStep1D^[p]) s) := by
  induction p with
  | zero => simp
  | succ p ih =>
    simp only [Function.iterate_succ, Function.comp]
    rw [ih, step_commutes_with_rotation]

-- Corollary: if s is a period-p oscillator, so is every rotation of it
theorem rotation_preserves_period (s : RingState) (p : Nat)
    (hp : (ringStep1D^[p]) s = s) :
    (ringStep1D^[p]) (rotate s) = rotate s := by
  rw [iter_commutes_with_rotation, hp]

-- ============================================================
-- Complementation symmetry: flip all bits
-- (dead↔alive symmetry of the rule B1/S12)
-- ============================================================

def complement (s : RingState) : RingState :=
  fun i => !s i

-- Does ringStep1D commute with complement?
-- Born if 1 neighbour (alive: 0 or 1 or 2 → survival criterion, dead: exactly 1 → born)
-- Complement flips alive/dead but neighbour count stays same.
-- Born condition: n=1 for dead cell ↔ survival condition: n=1 for alive cell ✓
-- Survival condition: n∈{1,2} for alive ↔ born condition n=1 for dead... not quite.
-- So complementation is NOT a symmetry of this rule in general.

-- Verify:
#eval! stateToN (ringStep1D (complement (nToState 0b00000001)))
#eval! stateToN (complement (ringStep1D (nToState 0b00000001)))
-- (expecting these to differ, confirming no complement symmetry)

-- ============================================================
-- The all-dead state is a fixed point
-- ============================================================

theorem all_dead_fixed : ringStep1D (fun _ => false) = fun _ => false := by
  funext i
  simp [ringStep1D, ringNeighbours]

-- The all-alive state on the 8-ring: what does it do?
#eval stateToN (ringStep1D (fun _ => true))
-- All cells have 2 neighbours → all survive (n=2 is in survival set {1,2})
-- So all-alive is also a fixed point!

theorem all_alive_fixed : ringStep1D (fun _ => true) = fun _ => true := by
  funext i
  simp [ringStep1D, ringNeighbours]

