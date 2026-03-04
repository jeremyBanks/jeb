-- GameOfLifeCorollaries.lean
-- Corollaries and extensions building on GameOfLife.lean
-- Using translation invariance and support theorems.

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

def isStillLife (g : Grid) : Prop := ∀ p, step g p = g p

def translate (g : Grid) (dx dy : Int) : Grid :=
  fun (x, y) => g (x - dx, y - dy)

-- Translation invariance (from GameOfLife.lean)
theorem step_translation_invariant (g : Grid) (tdx tdy : Int) :
    step (translate g tdx tdy) = translate (step g) tdx tdy := by
  funext ⟨x, y⟩
  simp only [step, translate, lifeRule, liveNeighbours]
  have ha : x - tdx - 1 = x - 1 - tdx := by omega
  have hb : x - tdx + 1 = x + 1 - tdx := by omega
  have hc : y - tdy - 1 = y - 1 - tdy := by omega
  have hd : y - tdy + 1 = y + 1 - tdy := by omega
  simp only [ha, hb, hc, hd]

-- ============================================================
-- Corollary 1: Translates of still lifes are still lifes
-- ============================================================

theorem translate_still_life {g : Grid} (hg : isStillLife g) (dx dy : Int) :
    isStillLife (translate g dx dy) := by
  intro ⟨x, y⟩
  -- step (translate g dx dy) = translate (step g) dx dy  [invariance]
  -- then translate (step g) dx dy (x,y) = (step g) (x-dx, y-dy)
  --                                      = g (x-dx, y-dy)            [hg]
  --                                      = (translate g dx dy) (x,y)
  show step (translate g dx dy) (x, y) = (translate g dx dy) (x, y)
  have hinv := step_translation_invariant g dx dy
  rw [hinv]
  simp only [translate]
  exact hg (x - dx, y - dy)

-- ============================================================
-- Corollary 2: Composition of translations
-- ============================================================

theorem translate_compose (g : Grid) (dx1 dy1 dx2 dy2 : Int) :
    translate (translate g dx1 dy1) dx2 dy2 = translate g (dx1 + dx2) (dy1 + dy2) := by
  funext ⟨x, y⟩
  simp only [translate]
  have h : (x - dx2 - dx1, y - dy2 - dy1) = (x - (dx1 + dx2), y - (dy1 + dy2)) := by
    simp only [Prod.mk.injEq]; exact ⟨by omega, by omega⟩
  rw [h]

theorem translate_zero (g : Grid) : translate g 0 0 = g := by
  funext ⟨x, y⟩; simp [translate]

-- ============================================================
-- Corollary 3: The set of still lifes is closed under the
-- translation group (arbitrary sequences of translations)
-- ============================================================

theorem translate_still_life_compose {g : Grid} (hg : isStillLife g)
    (dx1 dy1 dx2 dy2 : Int) :
    isStillLife (translate (translate g dx1 dy1) dx2 dy2) :=
  translate_still_life (translate_still_life hg dx1 dy1) dx2 dy2

-- The translation group is infinite — there are infinitely many distinct
-- translates of any non-empty still life (they're all still lifes too).

-- ============================================================
-- Corollary 4: step is local — depends only on 3×3 neighbourhood
-- ============================================================
-- If two grids agree on the 3×3 box around (cx,cy), step agrees at (cx,cy).

def agreesOn (g1 g2 : Grid) (x0 x1 y0 y1 : Int) : Prop :=
  ∀ x y : Int, x0 ≤ x → x ≤ x1 → y0 ≤ y → y ≤ y1 → g1 (x, y) = g2 (x, y)

-- Helper: equal bools give equal nat contributions
private theorem bool_eq_nat {b1 b2 : Bool} (h : b1 = b2) :
    (if b1 then 1 else 0) = (if b2 then 1 else 0) := by rw [h]

theorem step_local (g1 g2 : Grid) (cx cy : Int)
    (h : agreesOn g1 g2 (cx-1) (cx+1) (cy-1) (cy+1)) :
    step g1 (cx, cy) = step g2 (cx, cy) := by
  simp only [step, lifeRule, liveNeighbours]
  -- Replace each cell lookup with its equal counterpart
  have eq00 := h cx     cy     (by omega) (by omega) (by omega) (by omega)
  have eqm1m1 := h (cx-1) (cy-1) (by omega) (by omega) (by omega) (by omega)
  have eq0m1  := h cx     (cy-1) (by omega) (by omega) (by omega) (by omega)
  have eq1m1  := h (cx+1) (cy-1) (by omega) (by omega) (by omega) (by omega)
  have eqm10  := h (cx-1) cy     (by omega) (by omega) (by omega) (by omega)
  have eq10   := h (cx+1) cy     (by omega) (by omega) (by omega) (by omega)
  have eqm11  := h (cx-1) (cy+1) (by omega) (by omega) (by omega) (by omega)
  have eq01   := h cx     (cy+1) (by omega) (by omega) (by omega) (by omega)
  have eq11   := h (cx+1) (cy+1) (by omega) (by omega) (by omega) (by omega)
  rw [eq00, bool_eq_nat eqm1m1, bool_eq_nat eq0m1, bool_eq_nat eq1m1,
      bool_eq_nat eqm10, bool_eq_nat eq10, bool_eq_nat eqm11,
      bool_eq_nat eq01, bool_eq_nat eq11]

-- ============================================================
-- Corollary 5: The empty grid and full grid facts
-- ============================================================

def emptyGrid : Grid := fun _ => false
def fullGrid  : Grid := fun _ => true

theorem empty_is_still_life : isStillLife emptyGrid := by
  intro ⟨x, y⟩; simp [step, emptyGrid, lifeRule, liveNeighbours]

theorem full_grid_not_still_life : ¬ isStillLife fullGrid := by
  intro h
  have hmatch := h (0, 0)
  simp [step, fullGrid, lifeRule, liveNeighbours] at hmatch

-- ============================================================
-- Corollary 6: Distinct witnesses — block ≠ empty
-- ============================================================

def blockGrid : Grid := fun (x, y) =>
  (x == 0 || x == 1) && (y == 0 || y == 1)

theorem block_ne_empty : blockGrid ≠ emptyGrid := by
  intro h
  have := congrFun h (0, 0)
  simp [blockGrid, emptyGrid] at this

-- Every translate of block is also distinct from empty (since it's non-empty)
theorem translate_block_ne_empty (dx dy : Int) :
    translate blockGrid dx dy ≠ emptyGrid := by
  intro h
  have := congrFun h (dx, dy)
  simp [translate, blockGrid, emptyGrid] at this

-- ============================================================
-- Corollary 7: Iterated step — stepⁿ
-- ============================================================

-- Apply step n times to a grid
def stepN : Nat → Grid → Grid
  | 0,     g => g
  | n + 1, g => step (stepN n g)

theorem stepN_zero (g : Grid) : stepN 0 g = g := rfl
theorem stepN_succ (n : Nat) (g : Grid) : stepN (n + 1) g = step (stepN n g) := rfl

-- Still lifes are fixed points: stepⁿ g = g for all n
theorem still_life_iterated {g : Grid} (hg : isStillLife g) (n : Nat) :
    stepN n g = g := by
  induction n with
  | zero => rfl
  | succ n ih =>
      simp only [stepN_succ, ih]
      exact funext (fun p => hg p)

-- Translation invariance extends to iterated step
theorem stepN_translation_invariant (g : Grid) (n : Nat) (dx dy : Int) :
    stepN n (translate g dx dy) = translate (stepN n g) dx dy := by
  induction n with
  | zero => rfl
  | succ n ih =>
      simp only [stepN_succ, ih, step_translation_invariant]

-- ============================================================
-- Corollary 8: step is deterministic (trivial but explicit)
-- ============================================================

theorem step_deterministic (g : Grid) (p : Int × Int) :
    step g p = step g p := rfl
