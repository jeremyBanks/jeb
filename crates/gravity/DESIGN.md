# Partli: A Particle-Life Cellular Automaton

Partli combines Conway's Game of Life with a particle gravity simulation. Each live cell contains a particle with continuous position and velocity. The Conway rules govern birth and death, while gravity and movement create emergent physical behavior.

## Coordinate System

- 2D grid with integer dimensions `W` (width) and `H` (height)
- Origin (0, 0) at bottom-left
- Each cell spans a 1.0 × 1.0 floating-point region
- Cell (x, y) covers the region [x, x+1) × [y, y+1)
- Particle positions are floating-point coordinates within this space

## Grid Topology

The grid may wrap around its edges, creating a toroidal topology.

- `RA` (Wrap Enabled): When true, edges connect to opposite edges
- `SX` (Stagger X): When wrapping horizontally, the y-coordinate shifts by this amount
- `SY` (Stagger Y): When wrapping vertically, the x-coordinate shifts by this amount

Only one of `SX` or `SY` may be non-zero at a time.

**Wrapping behavior**: When a coordinate exceeds the grid bounds:
- If x < 0: x += W, y += SY
- If x ≥ W: x -= W, y -= SY  
- If y < 0: y += H, x += SX
- If y ≥ H: y -= H, x -= SX

This is applied repeatedly until the coordinate is within bounds.

When wrapping is disabled, coordinates outside the grid are invalid/inaccessible.

## State

Each cell has:
- **Alive**: Boolean, whether the cell contains a living particle
- **Position**: (x, y) float coordinates of the particle within the grid (only meaningful when alive)
- **Velocity**: (vx, vy) float velocity vector (only meaningful when alive)
- **Color**: Hue, saturation, and lightness values for rendering (persists briefly after death for fade effect)

## Initialization

- Each cell has probability `DN` (Initial Density, as percentage of total cells) of being alive
- Living particles are positioned at the center of their cell: (cellX + 0.5, cellY + 0.5)
- All initial velocities are (0, 0)

## Tick Processing

Each tick processes in this order:
1. Generate shuffled cell order
2. Conway phase (births and deaths)
3. Gravity phase (velocity changes)
4. Movement phase (position changes)
5. Color update phase

### Shuffled Cell Order

At the start of each tick, create a randomly shuffled list of all cell indices. This single ordering is used throughout the tick for any operation that requires iteration. When filtering for specific cells (e.g., only live cells, only dying cells), preserve the relative order from this global shuffle.

### Conway Phase

**Neighbor counting**: Each cell has up to 8 neighbors (orthogonal and diagonal). With wrapping enabled, neighbors are found via the wrapping rules. With wrapping disabled, cells at edges have fewer neighbors.

**Birth rule**: A dead cell with exactly 3 live neighbors will be born.

**Death rule**: A live cell with fewer than 2 or more than 3 live neighbors will die. 

**Exception**: If `UD` (Unconserved Deaths) is false, cells with exactly 0 neighbors are protected from death.

**Rate limiting**: The raw lists of births and deaths are filtered by several limits, applied in order:

1. `RB` (Birth Rate): Maximum births allowed (as count or percentage of current population)
2. `RD` (Death Rate): Maximum deaths allowed (as count or percentage of current population)
3. `RC` (Change Rate): Maximum total changes (births + deaths). While over limit, randomly remove from either list with equal probability per item.
4. `PI` (Min Population): While (population - deaths) < PI, remove deaths
5. `PA` (Max Population): While (population + births) > PA, remove births

The lists are processed in shuffled order, so which specific births/deaths are removed is random.

**Applying changes**: Births and deaths are interleaved according to their position in the global shuffled order, not processed as separate batches.

**Birth mechanics**:
- Count live neighbors at moment of birth: `count`
- Position: Average of neighbor particle positions, clamped to stay within the cell's bounds
- Velocity: For each neighbor, take `1/(count + 1)` of its velocity and subtract from neighbor. Sum these to get newborn's velocity. (Momentum-conserving)

**Death mechanics**:
- Count live neighbors at moment of death: `count`
- If `UD` is false and `count` is 0, the death is cancelled (cell survives)
- Otherwise: Divide dying particle's velocity by `count`, add this to each neighbor's velocity. (Momentum-conserving, may exceed speed limit)

### Gravity Phase

Every live particle exerts gravitational attraction on every other live particle.

**Force calculation**: For particles A and B with positions pA and pB:
```
dx = pB.x - pA.x
dy = pB.y - pA.y
dist² = dx² + dy²
dist = sqrt(dist² + GS²)
force = G / (dist² + GS²)
forceX = force * dx / dist
forceY = force * dy / dist
```

Where:
- `G` (Coefficient of Gravity): Strength of gravitational attraction
- `GS` (Gravity Softening): Prevents singularities at very close distances

**Wrapping and gravity**: When wrapping is enabled, each particle is attracted not only to other particles but also to their wrapped copies. For each other particle, consider the 5 nearest copies: the particle itself plus the 4 copies found by looking in each cardinal direction (up, down, left, right) into the adjacent wrapped grid spaces. Also, each particle is attracted to its own 4 wrapped copies.

When wrapping is disabled, only actual particle positions are used.

**Barnes-Hut approximation**: When `QT` > 0, a quadtree-based Barnes-Hut approximation is used instead of exact O(n²) calculation:
- Build a quadtree containing all particles (plus ghost copies at wrapped positions if wrapping)
- For each particle, traverse the tree. If a node's `size / distance < QT`, treat the entire node as a single mass at its center of mass. Otherwise, recurse into children.
- When `QT` = 0, use exact pairwise calculation

**Speed limiting**: Before applying gravity forces, record each particle's current speed. After applying forces, if the new speed exceeds `max(SL, old_speed)`, scale the velocity down to that limit. This means:
- Gravity alone cannot accelerate past `SL` (Speed Limit)
- Particles already moving faster than `SL` (from momentum transfer) maintain their speed until something slows them
- Direction can always change

### Movement Phase

Process particles in shuffled order. For each particle:

1. Calculate target position: current position + velocity
2. With wrapping enabled, wrap the target position into bounds
3. Without wrapping, if target is out of bounds, particle is stuck (cannot move)
4. Determine target cell via `floor(targetX)`, `floor(targetY)`
5. If target cell is the current cell, update position (movement within cell)
6. If target cell is occupied by another particle, the particle is stuck:
   - Add to a queue for that target cell
   - Do not move this tick
   - Velocity is preserved (will try again next tick)
7. If target cell is empty, move:
   - Mark old cell as dead/empty
   - Mark new cell as alive
   - Transfer position, velocity, and color state to new cell index
   - **Cascade**: Check if any particles were queued for the now-empty cell. If so, the first queued particle moves into it, potentially freeing another cell and triggering further cascades.

### Color Update Phase

For each cell:

**If alive**:
- Hue: Direction of velocity in degrees, `atan2(vy, vx) * 180/π`, normalized to [0, 360)
- Saturation: Based on speed relative to speed limit: `baseSaturation + (speed / (SL * 0.75)) * saturationRange`, capped at maximum
- Lightness: Higher if particle moved this tick, lower if stuck

**If dead but has residual color**:
- Apply fade: `L = L * (1 - 1/FP) - 1/FF`, clamped to 0
- Saturation fades similarly

Where:
- `FD` (Death Fade): Percentage to immediately reduce lightness on death (0% = no instant fade, 100% = instant black)
- `FP` (Fade Proportional): Divisor for proportional fade component (higher = slower fade)
- `FF` (Fade Fixed): Divisor for fixed fade component (higher = slower fade)

## Rendering

The grid is rendered to a 2D canvas.

**With wrapping enabled**: The canvas is 2W × 2H pixels. The "core" grid occupies the center W × H region. Surrounding it, wrapped copies of the grid are displayed:
- Surrounding cells (not on immediate border): Darkened by 25%
- Immediate border cells (1-pixel ring around core): Lightened by 25% instead of darkened
- Core cells: Rendered at full brightness

**With wrapping disabled**: The canvas is W × H pixels, showing only the core grid.

**Color transformation**:
- A base hue offset is added to all particle hues before conversion to RGB
- Colors are stored and processed in a perceptual color space (OKLCH or similar)
- Final conversion to RGB happens at render time

**Rotation**: The rendered output may be rotated by a configurable angle. This is a pure display transformation that doesn't affect simulation.

**Scaling**: The canvas is displayed at an integer multiple of its native size when small, or scaled down smoothly when large, to fit a target display area.

## Parameters Summary

### Grid Settings
| Parameter | Name | Default | Description |
|-----------|------|---------|-------------|
| W | Width | 192 | Grid width in cells |
| H | Height | 120 | Grid height in cells |
| RA | Wrap Enabled | true | Whether edges wrap |
| SX | Stagger X | 96* | X offset when wrapping vertically |
| SY | Stagger Y | 0* | Y offset when wrapping horizontally |
| DN | Initial Density | ~0.78% | Percentage of cells initially alive |

*Default stagger is half the larger even dimension, or 0 if equal/both odd.

### Population Limits
| Parameter | Name | Default | Description |
|-----------|------|---------|-------------|
| PI | Min Population | 124 | Minimum population (deaths cancelled below this) |
| PA | Max Population | 132 | Maximum population (births cancelled above this) |

### Rate Limits
| Parameter | Name | Default | Description |
|-----------|------|---------|-------------|
| RB | Birth Rate | 4 | Maximum births per tick |
| RD | Death Rate | 4 | Maximum deaths per tick |
| RC | Change Rate | 4 | Maximum total changes per tick |

### Particle Physics
| Parameter | Name | Default | Description |
|-----------|------|---------|-------------|
| G | Gravity | 0.5 | Gravitational constant |
| GS | Gravity Softening | 2.5 | Softening factor to prevent singularities |
| SL | Speed Limit | 2.25 | Maximum speed from gravity acceleration |
| UD | Unconserved Deaths | false | Whether 0-neighbor cells can die |
| QT | Approximation | 0.5 | Barnes-Hut theta (0 = exact calculation) |

### Cosmetic
| Parameter | Name | Default | Description |
|-----------|------|---------|-------------|
| Hue | Base Hue | 165° | Added to all particle hues |
| Rotation | Rotation | 0° | Display rotation angle |
| FD | Death Fade | 75% | Instant brightness reduction on death |
| FP | Fade Proportional | 6 | Proportional fade divisor |
| FF | Fade Fixed | 127 | Fixed fade divisor |

## Invariants and Edge Cases

- Momentum is conserved through births and deaths (within floating-point precision)
- Energy is NOT conserved (gravity adds energy, speed limit removes it)
- A particle stuck trying to move out of bounds retains its velocity indefinitely until redirected
- A particle stuck behind another may suddenly move via cascade when the blocking particle moves
- Very small populations may stabilize into static or oscillating patterns due to Conway rules
- With `UD` = false, isolated particles (0 neighbors) are immortal
