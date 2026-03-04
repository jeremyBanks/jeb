-- LifeSymmetry.lean
-- Life's step function commutes with the 8 symmetries of D₄
-- (reflections across both axes, 180° rotation, and diagonal).
-- Strategy: unfold neighbour sums, rewrite coordinates by arithmetic,
-- then use omega (which handles commutative Nat addition) to close.

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

-- ============================================================
-- Symmetry operations
-- ============================================================

def reflX    (g : Grid) : Grid := fun (x, y) => g (-x,  y)
def reflY    (g : Grid) : Grid := fun (x, y) => g ( x, -y)
def rot180   (g : Grid) : Grid := fun (x, y) => g (-x, -y)
def transp   (g : Grid) : Grid := fun (x, y) => g ( y,  x)  -- diagonal reflection

-- ============================================================
-- Helper: pair equality by components
-- ============================================================

private theorem pair_eq {a b c d : Int} (h1 : a = c) (h2 : b = d) : (a, b) = (c, d) := by
  subst h1; subst h2; rfl

-- ============================================================
-- Neighbour count symmetries
-- After unfolding, both sides are the same 8 terms summed in
-- different orders — omega handles commutative addition.
-- ============================================================

theorem liveN_reflX (g : Grid) (x y : Int) :
    liveNeighbours (reflX g) x y = liveNeighbours g (-x) y := by
  simp only [liveNeighbours, reflX]
  have h1 : (-(x-1), y-1) = (-x+1, y-1) := pair_eq (by omega) rfl
  have h2 : (-(x+1), y-1) = (-x-1, y-1) := pair_eq (by omega) rfl
  have h3 : (-(x-1), y  ) = (-x+1, y  ) := pair_eq (by omega) rfl
  have h4 : (-(x+1), y  ) = (-x-1, y  ) := pair_eq (by omega) rfl
  have h5 : (-(x-1), y+1) = (-x+1, y+1) := pair_eq (by omega) rfl
  have h6 : (-(x+1), y+1) = (-x-1, y+1) := pair_eq (by omega) rfl
  -- simp can rewrite through if-then-else; rw cannot (motive not type correct)
  simp only [h1, h2, h3, h4, h5, h6]; omega

theorem liveN_reflY (g : Grid) (x y : Int) :
    liveNeighbours (reflY g) x y = liveNeighbours g x (-y) := by
  simp only [liveNeighbours, reflY]
  have h1 : (x-1, -(y-1)) = (x-1, -y+1) := pair_eq rfl (by omega)
  have h2 : (x,   -(y-1)) = (x,   -y+1) := pair_eq rfl (by omega)
  have h3 : (x+1, -(y-1)) = (x+1, -y+1) := pair_eq rfl (by omega)
  have h4 : (x-1, -(y+1)) = (x-1, -y-1) := pair_eq rfl (by omega)
  have h5 : (x,   -(y+1)) = (x,   -y-1) := pair_eq rfl (by omega)
  have h6 : (x+1, -(y+1)) = (x+1, -y-1) := pair_eq rfl (by omega)
  simp only [h1, h2, h3, h4, h5, h6]; omega

theorem liveN_rot180 (g : Grid) (x y : Int) :
    liveNeighbours (rot180 g) x y = liveNeighbours g (-x) (-y) := by
  simp only [liveNeighbours, rot180]
  have h1 : (-(x-1), -(y-1)) = (-x+1, -y+1) := pair_eq (by omega) (by omega)
  have h2 : (-x,     -(y-1)) = (-x,   -y+1) := pair_eq rfl        (by omega)
  have h3 : (-(x+1), -(y-1)) = (-x-1, -y+1) := pair_eq (by omega) (by omega)
  have h4 : (-(x-1), -y    ) = (-x+1, -y  ) := pair_eq (by omega) rfl
  have h5 : (-(x+1), -y    ) = (-x-1, -y  ) := pair_eq (by omega) rfl
  have h6 : (-(x-1), -(y+1)) = (-x+1, -y-1) := pair_eq (by omega) (by omega)
  have h7 : (-x,     -(y+1)) = (-x,   -y-1) := pair_eq rfl        (by omega)
  have h8 : (-(x+1), -(y+1)) = (-x-1, -y-1) := pair_eq (by omega) (by omega)
  simp only [h1, h2, h3, h4, h5, h6, h7, h8]; omega

-- Transpose: the 8 neighbours of (x,y) with coords swapped = 8 neighbours of (y,x)
-- Both sides have the same 8 grid lookups in different sum order → omega
theorem liveN_transp (g : Grid) (x y : Int) :
    liveNeighbours (transp g) x y = liveNeighbours g y x := by
  simp only [liveNeighbours, transp]; omega

-- ============================================================
-- step commutes with each symmetry
-- ============================================================

theorem step_reflX_comm (g : Grid) : step (reflX g) = reflX (step g) := by
  funext ⟨x, y⟩; simp only [step, reflX, lifeRule, liveN_reflX]

theorem step_reflY_comm (g : Grid) : step (reflY g) = reflY (step g) := by
  funext ⟨x, y⟩; simp only [step, reflY, lifeRule, liveN_reflY]

theorem step_rot180_comm (g : Grid) : step (rot180 g) = rot180 (step g) := by
  funext ⟨x, y⟩; simp only [step, rot180, lifeRule, liveN_rot180]

theorem step_transp_comm (g : Grid) : step (transp g) = transp (step g) := by
  funext ⟨x, y⟩; simp only [step, transp, lifeRule, liveN_transp]

-- ============================================================
-- 90° rotation = transp ∘ reflX; 270° = rot90 ∘ rot180
-- ============================================================

def rot90  (g : Grid) : Grid := transp (reflX g)
def rot270 (g : Grid) : Grid := rot90 (rot180 g)

theorem step_rot90_comm (g : Grid) : step (rot90 g) = rot90 (step g) := by
  simp only [rot90, ← step_transp_comm, ← step_reflX_comm]

theorem step_rot270_comm (g : Grid) : step (rot270 g) = rot270 (step g) := by
  simp only [rot270, ← step_rot90_comm, ← step_rot180_comm]

-- ============================================================
-- Symmetry is preserved under evolution
-- ============================================================

def stepN : Nat → Grid → Grid
  | 0,     g => g
  | n + 1, g => step (stepN n g)

-- If a pattern has a symmetry, all future generations have it too.
-- Proof: induction + step commutes with the symmetry operation.

theorem reflX_symm_preserved {g : Grid} (h : reflX g = g) (n : Nat) :
    reflX (stepN n g) = stepN n g := by
  induction n with
  | zero => simpa
  | succ n ih => simp only [stepN, ← step_reflX_comm, ih]

theorem reflY_symm_preserved {g : Grid} (h : reflY g = g) (n : Nat) :
    reflY (stepN n g) = stepN n g := by
  induction n with
  | zero => simpa
  | succ n ih => simp only [stepN, ← step_reflY_comm, ih]

theorem rot180_symm_preserved {g : Grid} (h : rot180 g = g) (n : Nat) :
    rot180 (stepN n g) = stepN n g := by
  induction n with
  | zero => simpa
  | succ n ih => simp only [stepN, ← step_rot180_comm, ih]

theorem transp_symm_preserved {g : Grid} (h : transp g = g) (n : Nat) :
    transp (stepN n g) = stepN n g := by
  induction n with
  | zero => simpa
  | succ n ih => simp only [stepN, ← step_transp_comm, ih]

-- ============================================================
-- The empty grid and full grid have all D₄ symmetries
-- ============================================================

def emptyGrid : Grid := fun _ => false

theorem emptyGrid_reflX : reflX emptyGrid = emptyGrid := funext (fun _ => rfl)
theorem emptyGrid_rot180 : rot180 emptyGrid = emptyGrid := funext (fun _ => rfl)
theorem emptyGrid_transp : transp emptyGrid = emptyGrid := funext (fun _ => rfl)

-- The blinker: blinkerH and blinkerV are related by transpose
-- (since transposing swaps x and y, turning horizontal into vertical)
def blinkerH : Grid := fun (x, y) => y == 0 && (x == -1 || x == 0 || x == 1)
def blinkerV : Grid := fun (x, y) => x == 0 && (y == -1 || y == 0 || y == 1)

theorem blinkerH_transp_eq_blinkerV : transp blinkerH = blinkerV := by
  funext ⟨x, y⟩
  simp [transp, blinkerH, blinkerV]
