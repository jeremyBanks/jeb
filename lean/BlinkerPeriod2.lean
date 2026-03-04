-- BlinkerPeriod2.lean
-- Formal proof that the Life blinker is a period-2 oscillator.

def Grid := Int × Int → Bool

def liveNeighbours (g : Grid) (x y : Int) : Nat :=
  (if g (x-1, y-1) then 1 else 0) + (if g (x,   y-1) then 1 else 0) +
  (if g (x+1, y-1) then 1 else 0) + (if g (x-1, y  ) then 1 else 0) +
  (if g (x+1, y  ) then 1 else 0) + (if g (x-1, y+1) then 1 else 0) +
  (if g (x,   y+1) then 1 else 0) + (if g (x+1, y+1) then 1 else 0)

def lifeRule (alive : Bool) (n : Nat) : Bool :=
  if alive then n == 2 || n == 3 else n == 3

def step (g : Grid) : Grid :=
  fun (x, y) => lifeRule (g (x, y)) (liveNeighbours g x y)

def blinkerH : Grid := fun (x, y) => y == 0 && (x == -1 || x == 0 || x == 1)
def blinkerV : Grid := fun (x, y) => x == 0 && (y == -1 || y == 0 || y == 1)

-- ============================================================
-- Support lemmas: each blinker is dead outside its 3-cell strip
-- ============================================================

theorem blinkerH_support (x y : Int) (hout : x < -1 ∨ x > 1 ∨ y ≠ 0) :
    blinkerH (x, y) = false := by
  simp only [blinkerH, Bool.and_eq_false_iff]
  by_cases hy : y = 0
  · subst hy
    right
    simp only [Bool.or_eq_false_iff, beq_eq_false_iff_ne]
    rcases hout with h | h | h
    · exact ⟨⟨by omega, by omega⟩, by omega⟩
    · exact ⟨⟨by omega, by omega⟩, by omega⟩
    · exact absurd rfl h
  · left; exact beq_eq_false_iff_ne.mpr hy

theorem blinkerV_support (x y : Int) (hout : x ≠ 0 ∨ y < -1 ∨ y > 1) :
    blinkerV (x, y) = false := by
  simp only [blinkerV, Bool.and_eq_false_iff]
  by_cases hx : x = 0
  · subst hx
    right
    simp only [Bool.or_eq_false_iff, beq_eq_false_iff_ne]
    rcases hout with h | h | h
    · exact absurd rfl h
    · exact ⟨⟨by omega, by omega⟩, by omega⟩
    · exact ⟨⟨by omega, by omega⟩, by omega⟩
  · left; exact beq_eq_false_iff_ne.mpr hx

-- ============================================================
-- Neighbourhood lemmas: far cells have 0 live neighbours
-- ============================================================

-- Helper: dead cell contributes 0 to neighbour sum
private theorem dead_contrib {g : Grid} {nx ny : Int} (h : g (nx, ny) = false) :
    (if g (nx, ny) then 1 else 0) = 0 := by simp [h]

theorem blinkerH_far_zero_neighbours (x y : Int)
    (hfar : x < -2 ∨ x > 2 ∨ y < -1 ∨ y > 1) :
    liveNeighbours blinkerH x y = 0 := by
  simp only [liveNeighbours]
  have kill : ∀ nx ny : Int, (nx < -1 ∨ nx > 1 ∨ ny ≠ 0) →
      (if blinkerH (nx, ny) then 1 else 0) = 0 := fun nx ny h =>
    dead_contrib (blinkerH_support nx ny h)
  simp [kill (x-1) (y-1) (by omega), kill x (y-1) (by omega),
        kill (x+1) (y-1) (by omega), kill (x-1) y (by omega),
        kill (x+1) y (by omega), kill (x-1) (y+1) (by omega),
        kill x (y+1) (by omega), kill (x+1) (y+1) (by omega)]

theorem blinkerV_far_zero_neighbours (x y : Int)
    (hfar : x < -1 ∨ x > 1 ∨ y < -2 ∨ y > 2) :
    liveNeighbours blinkerV x y = 0 := by
  simp only [liveNeighbours]
  have kill : ∀ nx ny : Int, (nx ≠ 0 ∨ ny < -1 ∨ ny > 1) →
      (if blinkerV (nx, ny) then 1 else 0) = 0 := fun nx ny h =>
    dead_contrib (blinkerV_support nx ny h)
  simp [kill (x-1) (y-1) (by omega), kill x (y-1) (by omega),
        kill (x+1) (y-1) (by omega), kill (x-1) y (by omega),
        kill (x+1) y (by omega), kill (x-1) (y+1) (by omega),
        kill x (y+1) (by omega), kill (x+1) (y+1) (by omega)]

-- ============================================================
-- Far cells: step = false = other blinker
-- ============================================================

theorem blinkerH_far_step_eq_blinkerV (x y : Int)
    (hfar : x < -2 ∨ x > 2 ∨ y < -1 ∨ y > 1) :
    step blinkerH (x, y) = blinkerV (x, y) := by
  have h0 := blinkerH_far_zero_neighbours x y hfar
  have hd := blinkerH_support x y (by omega)
  have hv := blinkerV_support x y (by omega)
  simp [step, lifeRule, h0, hd, hv]

theorem blinkerV_far_step_eq_blinkerH (x y : Int)
    (hfar : x < -1 ∨ x > 1 ∨ y < -2 ∨ y > 2) :
    step blinkerV (x, y) = blinkerH (x, y) := by
  have h0 := blinkerV_far_zero_neighbours x y hfar
  have hd := blinkerV_support x y (by omega)
  have hh := blinkerH_support x y (by omega)
  simp [step, lifeRule, h0, hd, hh]

-- ============================================================
-- Near cells: verified by native_decide on the 15/25 finite cases
-- ============================================================

theorem blinkerH_near_step_eq_blinkerV (x y : Int)
    (hx : -2 ≤ x ∧ x ≤ 2) (hy : -1 ≤ y ∧ y ≤ 1) :
    step blinkerH (x, y) = blinkerV (x, y) := by
  have hxv : x = -2 ∨ x = -1 ∨ x = 0 ∨ x = 1 ∨ x = 2 := by omega
  have hyv : y = -1 ∨ y = 0 ∨ y = 1 := by omega
  rcases hxv with rfl|rfl|rfl|rfl|rfl <;> rcases hyv with rfl|rfl|rfl <;> native_decide

theorem blinkerV_near_step_eq_blinkerH (x y : Int)
    (hx : -1 ≤ x ∧ x ≤ 1) (hy : -2 ≤ y ∧ y ≤ 2) :
    step blinkerV (x, y) = blinkerH (x, y) := by
  have hxv : x = -1 ∨ x = 0 ∨ x = 1 := by omega
  have hyv : y = -2 ∨ y = -1 ∨ y = 0 ∨ y = 1 ∨ y = 2 := by omega
  rcases hxv with rfl|rfl|rfl <;> rcases hyv with rfl|rfl|rfl|rfl|rfl <;> native_decide

-- ============================================================
-- Main theorems
-- ============================================================

theorem step_blinkerH_eq_blinkerV : step blinkerH = blinkerV := by
  funext ⟨x, y⟩
  by_cases hx1 : x < -2
  · exact blinkerH_far_step_eq_blinkerV x y (Or.inl hx1)
  by_cases hx2 : x > 2
  · exact blinkerH_far_step_eq_blinkerV x y (Or.inr (Or.inl hx2))
  by_cases hy1 : y < -1
  · exact blinkerH_far_step_eq_blinkerV x y (Or.inr (Or.inr (Or.inl hy1)))
  by_cases hy2 : y > 1
  · exact blinkerH_far_step_eq_blinkerV x y (Or.inr (Or.inr (Or.inr hy2)))
  · exact blinkerH_near_step_eq_blinkerV x y ⟨by omega, by omega⟩ ⟨by omega, by omega⟩

theorem step_blinkerV_eq_blinkerH : step blinkerV = blinkerH := by
  funext ⟨x, y⟩
  by_cases hx1 : x < -1
  · exact blinkerV_far_step_eq_blinkerH x y (Or.inl hx1)
  by_cases hx2 : x > 1
  · exact blinkerV_far_step_eq_blinkerH x y (Or.inr (Or.inl hx2))
  by_cases hy1 : y < -2
  · exact blinkerV_far_step_eq_blinkerH x y (Or.inr (Or.inr (Or.inl hy1)))
  by_cases hy2 : y > 2
  · exact blinkerV_far_step_eq_blinkerH x y (Or.inr (Or.inr (Or.inr hy2)))
  · exact blinkerV_near_step_eq_blinkerH x y ⟨by omega, by omega⟩ ⟨by omega, by omega⟩

-- ============================================================
-- Period 2
-- ============================================================

def stepN : Nat → Grid → Grid
  | 0,     g => g
  | n + 1, g => step (stepN n g)

theorem blinker_period2 : stepN 2 blinkerH = blinkerH := by
  simp only [stepN]
  rw [step_blinkerH_eq_blinkerV, step_blinkerV_eq_blinkerH]

theorem blinker_not_still_life : step blinkerH ≠ blinkerH := by
  intro h
  have := congrFun h (-1, 0)
  rw [step_blinkerH_eq_blinkerV] at this
  simp [blinkerH, blinkerV] at this

theorem blinker_exact_period2 :
    stepN 1 blinkerH ≠ blinkerH ∧ stepN 2 blinkerH = blinkerH :=
  ⟨by simp [stepN]; exact blinker_not_still_life, blinker_period2⟩
