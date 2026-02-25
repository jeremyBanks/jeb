//! # Partli: A Particle-Life Cellular Automaton
//!
//! Partli combines Conway's Game of Life with particle gravity simulation.
//! Each live cell contains a particle with continuous position and velocity.
//! Conway rules govern birth and death; gravity creates emergent physical behavior.
//!
//! ## Key Invariants
//!
//! - **Momentum Conservation**: Total momentum is conserved through births and deaths
//! - **One Particle Per Cell**: At most one particle occupies each grid cell
//! - **Toroidal Wrapping**: Grid edges wrap with optional stagger offset
//!
//! ## Quick Example
//!
//! ```rust
//! use partli::{GridConfig, Partli, Vec2};
//!
//! // Create a 10x10 grid
//! let config = GridConfig::new(10, 10);
//! let mut sim = Partli::new(config);
//!
//! // Check initial state
//! assert_eq!(sim.population(), 0);
//! assert_eq!(sim.total_momentum(), Vec2::zero());
//!
//! // Birth a particle at cell (5, 5)
//! sim.birth_particle(5, 5);
//! assert_eq!(sim.population(), 1);
//! ```
//!
//! ## Design
//!
//! See `DESIGN.md` in this crate for the complete specification, including:
//! - Coordinate system and topology
//! - Tick processing phases (Conway, Gravity, Movement, Color)
//! - Parameter reference table
//! - Edge cases and invariants
//!
//! ## Testing
//!
//! This crate includes property-based tests verifying the momentum invariant
//! across random birth/death sequences. Run with `cargo test`.



/// 2D vector for positions and velocities
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

/// State of a single cell
#[derive(Debug, Clone)]
pub struct Cell {
    pub alive: bool,
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Cell {
    pub fn dead() -> Self {
        Self {
            alive: false,
            position: Vec2::zero(),
            velocity: Vec2::zero(),
        }
    }

    pub fn alive_at(pos: Vec2, vel: Vec2) -> Self {
        Self {
            alive: true,
            position: pos,
            velocity: vel,
        }
    }
}

/// Grid configuration
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
    pub wrap_enabled: bool,
    pub stagger_x: i32,
    pub stagger_y: i32,
}

impl GridConfig {
    pub fn new(width: usize, height: usize) -> Self {
        let stagger_x = if width > height && width % 2 == 0 {
            (width / 2) as i32
        } else {
            0
        };
        Self {
            width,
            height,
            wrap_enabled: true,
            stagger_x,
            stagger_y: 0,
        }
    }

    /// Wrap coordinates according to the stagger rules
    pub fn wrap(&self, mut x: f64, mut y: f64) -> (f64, f64) {
        if !self.wrap_enabled {
            return (x, y);
        }

        let w = self.width as f64;
        let h = self.height as f64;
        let sx = self.stagger_x as f64;
        let sy = self.stagger_y as f64;

        // Apply wrapping repeatedly until in bounds
        for _ in 0..10 {
            let mut changed = false;
            if x < 0.0 {
                x += w;
                y += sy;
                changed = true;
            }
            if x >= w {
                x -= w;
                y -= sy;
                changed = true;
            }
            if y < 0.0 {
                y += h;
                x += sx;
                changed = true;
            }
            if y >= h {
                y -= h;
                x -= sx;
                changed = true;
            }
            if !changed {
                break;
            }
        }

        // Final mod to ensure bounds
        x = ((x % w) + w) % w;
        y = ((y % h) + h) % h;

        (x, y)
    }

    /// Get cell coordinates from position
    pub fn cell_coords(&self, pos: Vec2) -> (usize, usize) {
        let (wx, wy) = if self.wrap_enabled {
            self.wrap(pos.x, pos.y)
        } else {
            (pos.x, pos.y)
        };
        (wx.floor() as usize, wy.floor() as usize)
    }

    /// Get cell index from (x, y) cell coordinates
    pub fn cell_index(&self, cx: usize, cy: usize) -> usize {
        cy * self.width + cx
    }

    /// Get neighbor cell coordinates (up to 8 neighbors)
    pub fn neighbors(&self, cx: usize, cy: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(8);
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;

                if self.wrap_enabled {
                    let (wx, wy) = self.wrap(nx as f64 + 0.5, ny as f64 + 0.5);
                    result.push((wx.floor() as usize, wy.floor() as usize));
                } else if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    result.push((nx as usize, ny as usize));
                }
            }
        }
        result
    }
}

/// The Partli simulation grid
#[derive(Debug, Clone)]
pub struct Partli {
    pub config: GridConfig,
    pub cells: Vec<Cell>,
    pub tick: u64,
}

impl Partli {
    pub fn new(config: GridConfig) -> Self {
        let total = config.width * config.height;
        let cells = vec![Cell::dead(); total];
        Self { config, cells, tick: 0 }
    }

    /// Initialize with random particles at given density (0.0 to 1.0)
    pub fn initialize_random(&mut self, density: f64, rng: &mut impl rand::Rng) {
        for cy in 0..self.config.height {
            for cx in 0..self.config.width {
                let idx = self.config.cell_index(cx, cy);
                if rng.gen::<f64>() < density {
                    self.cells[idx] = Cell::alive_at(
                        Vec2::new(cx as f64 + 0.5, cy as f64 + 0.5),
                        Vec2::zero(),
                    );
                } else {
                    self.cells[idx] = Cell::dead();
                }
            }
        }
    }

    /// Count live neighbors for a cell
    pub fn count_live_neighbors(&self, cx: usize, cy: usize) -> usize {
        self.config.neighbors(cx, cy)
            .iter()
            .filter(|(nx, ny)| {
                let idx = self.config.cell_index(*nx, *ny);
                self.cells[idx].alive
            })
            .count()
    }

    /// Get indices of live neighbors
    pub fn live_neighbor_indices(&self, cx: usize, cy: usize) -> Vec<usize> {
        self.config.neighbors(cx, cy)
            .iter()
            .filter_map(|(nx, ny)| {
                let idx = self.config.cell_index(*nx, *ny);
                if self.cells[idx].alive {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculate total momentum of all live particles
    pub fn total_momentum(&self) -> Vec2 {
        let mut total = Vec2::zero();
        for cell in &self.cells {
            if cell.alive {
                total += cell.velocity;
            }
        }
        total
    }

    /// Count live particles
    pub fn population(&self) -> usize {
        self.cells.iter().filter(|c| c.alive).count()
    }

    /// Execute a single birth with momentum conservation
    /// Returns true if birth succeeded
    pub fn birth_particle(&mut self, cx: usize, cy: usize) -> bool {
        let idx = self.config.cell_index(cx, cy);
        if self.cells[idx].alive {
            return false;
        }

        let neighbors = self.live_neighbor_indices(cx, cy);
        let count = neighbors.len();

        if count == 0 {
            // Birth with no neighbors: place at cell center with zero velocity
            self.cells[idx] = Cell::alive_at(
                Vec2::new(cx as f64 + 0.5, cy as f64 + 0.5),
                Vec2::zero(),
            );
            return true;
        }

        // Calculate average position of neighbors
        let mut avg_pos = Vec2::zero();
        for &n_idx in &neighbors {
            avg_pos += self.cells[n_idx].position;
        }
        avg_pos = avg_pos * (1.0 / count as f64);

        // Clamp to cell bounds
        let min_x = cx as f64;
        let max_x = cx as f64 + 0.9999;
        let min_y = cy as f64;
        let max_y = cy as f64 + 0.9999;
        avg_pos.x = avg_pos.x.clamp(min_x, max_x);
        avg_pos.y = avg_pos.y.clamp(min_y, max_y);

        // Momentum transfer: each neighbor loses 1/(count+1) of velocity
        let fraction = 1.0 / (count + 1) as f64;
        let mut new_velocity = Vec2::zero();
        for &n_idx in &neighbors {
            let take = self.cells[n_idx].velocity * fraction;
            self.cells[n_idx].velocity -= take;
            new_velocity += take;
        }

        self.cells[idx] = Cell::alive_at(avg_pos, new_velocity);
        true
    }

    /// Execute a single death with momentum conservation
    /// Returns true if death succeeded
    pub fn kill_particle(&mut self, cx: usize, cy: usize, unconserved_deaths: bool) -> bool {
        let idx = self.config.cell_index(cx, cy);
        if !self.cells[idx].alive {
            return false;
        }

        let neighbors = self.live_neighbor_indices(cx, cy);
        let count = neighbors.len();

        // If UD=false and no neighbors, death is cancelled
        if !unconserved_deaths && count == 0 {
            return false;
        }

        // Distribute velocity to neighbors
        if count > 0 {
            let share = self.cells[idx].velocity * (1.0 / count as f64);
            for &n_idx in &neighbors {
                self.cells[n_idx].velocity += share;
            }
        }

        self.cells[idx].alive = false;
        self.cells[idx].velocity = Vec2::zero();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    /// Test that wrap is idempotent
    #[test]
    fn test_wrap_idempotent() {
        let config = GridConfig::new(10, 10);
        for x in -20..30 {
            for y in -20..30 {
                let (x1, y1) = config.wrap(x as f64, y as f64);
                let (x2, y2) = config.wrap(x1, y1);
                assert!((x1 - x2).abs() < 1e-10, "x: {} -> {} -> {}", x, x1, x2);
                assert!((y1 - y2).abs() < 1e-10, "y: {} -> {} -> {}", y, y1, y2);
            }
        }
    }

    /// Test that wrapped coordinates are in bounds
    #[test]
    fn test_wrap_in_bounds() {
        let config = GridConfig::new(10, 10);
        for x in -50..60 {
            for y in -50..60 {
                let (wx, wy) = config.wrap(x as f64, y as f64);
                assert!(wx >= 0.0 && wx < 10.0, "x {} wrapped to {} out of bounds", x, wx);
                assert!(wy >= 0.0 && wy < 10.0, "y {} wrapped to {} out of bounds", y, wy);
            }
        }
    }

    /// Test that birth conserves momentum
    #[test]
    fn test_birth_momentum_conservation() {
        let mut grid = Partli::new(GridConfig::new(5, 5));

        // Set up 3 live neighbors with known velocities
        // Cell (2,2) will be born, neighbors at (1,1), (2,1), (3,1)
        grid.cells[grid.config.cell_index(1, 1)] = Cell::alive_at(
            Vec2::new(1.5, 1.5),
            Vec2::new(1.0, 0.0),
        );
        grid.cells[grid.config.cell_index(2, 1)] = Cell::alive_at(
            Vec2::new(2.5, 1.5),
            Vec2::new(0.0, 1.0),
        );
        grid.cells[grid.config.cell_index(3, 1)] = Cell::alive_at(
            Vec2::new(3.5, 1.5),
            Vec2::new(-1.0, 0.0),
        );

        let momentum_before = grid.total_momentum();
        grid.birth_particle(2, 2);
        let momentum_after = grid.total_momentum();

        let diff = (momentum_after - momentum_before).magnitude();
        assert!(diff < 1e-10, "Momentum not conserved: diff = {}", diff);
    }

    /// Test that death conserves momentum (when neighbors exist)
    #[test]
    fn test_death_momentum_conservation() {
        let mut grid = Partli::new(GridConfig::new(5, 5));

        // Set up particle at (2,2) with neighbors
        grid.cells[grid.config.cell_index(2, 2)] = Cell::alive_at(
            Vec2::new(2.5, 2.5),
            Vec2::new(2.0, 3.0),
        );
        grid.cells[grid.config.cell_index(1, 1)] = Cell::alive_at(
            Vec2::new(1.5, 1.5),
            Vec2::new(0.0, 0.0),
        );
        grid.cells[grid.config.cell_index(3, 3)] = Cell::alive_at(
            Vec2::new(3.5, 3.5),
            Vec2::new(1.0, 1.0),
        );

        let momentum_before = grid.total_momentum();
        grid.kill_particle(2, 2, true);
        let momentum_after = grid.total_momentum();

        let diff = (momentum_after - momentum_before).magnitude();
        assert!(diff < 1e-10, "Momentum not conserved: diff = {}", diff);
    }

    /// Test that death with UD=false and no neighbors is cancelled
    #[test]
    fn test_death_cancelled_isolated() {
        let mut grid = Partli::new(GridConfig::new(5, 5));

        // Isolated particle
        grid.cells[grid.config.cell_index(2, 2)] = Cell::alive_at(
            Vec2::new(2.5, 2.5),
            Vec2::new(1.0, 1.0),
        );

        let pop_before = grid.population();
        let killed = grid.kill_particle(2, 2, false);

        assert!(!killed, "Isolated particle should not die with UD=false");
        assert_eq!(grid.population(), pop_before);
    }

    /// Test that death with UD=true kills isolated particles
    #[test]
    fn test_death_allowed_isolated() {
        let mut grid = Partli::new(GridConfig::new(5, 5));

        // Isolated particle
        grid.cells[grid.config.cell_index(2, 2)] = Cell::alive_at(
            Vec2::new(2.5, 2.5),
            Vec2::new(1.0, 1.0),
        );

        let pop_before = grid.population();
        let killed = grid.kill_particle(2, 2, true);

        assert!(killed, "Isolated particle should die with UD=true");
        assert_eq!(grid.population(), pop_before - 1);
    }

    /// Property test: random births and deaths should conserve momentum
    #[test]
    fn test_random_births_deaths_momentum() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut grid = Partli::new(GridConfig::new(10, 10));
        grid.initialize_random(0.3, &mut rng);

        for _ in 0..100 {
            let momentum_before = grid.total_momentum();

            // Random operation
            let cx = rng.gen_range(0..10);
            let cy = rng.gen_range(0..10);

            if grid.cells[grid.config.cell_index(cx, cy)].alive {
                // Try to kill - only succeeds if has neighbors (conserves momentum)
                let neighbors = grid.live_neighbor_indices(cx, cy);
                if !neighbors.is_empty() {
                    grid.kill_particle(cx, cy, false);
                }
            } else {
                // Try to birth - conserves momentum via velocity transfer
                let neighbors = grid.count_live_neighbors(cx, cy);
                if neighbors == 3 {
                    grid.birth_particle(cx, cy);
                }
            }

            let momentum_after = grid.total_momentum();
            let diff = (momentum_after - momentum_before).magnitude();

            // Allow small floating point error
            assert!(
                diff < 1e-9,
                "Momentum change {} at iteration with momentum before {:?}, after {:?}",
                diff, momentum_before, momentum_after
            );
        }
    }
}

// [recovery] edit target not found, appending:
/// 2D vector
