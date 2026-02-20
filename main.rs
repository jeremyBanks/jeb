use std::fs;
use std::io::{BufWriter, Write};
use std::process::Command;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

const W: usize = 192;
const H: usize = 120; // 192×120 × 10 = 1920×1200 exactly (square pixels)

// Output video settings
const OUT_W: u32 = 1920; // 192 × 10
const OUT_H: u32 = 1200; // 120 × 10
const FPS: u32 = 60;
const CRF: u32 = 12;
const CHUNK_FRAMES: usize = 3840; // 64s at 60fps

struct Cell {
    x: usize,  // integer grid column [0, W)
    y: usize,  // integer grid row    [0, H)
    vx: f32,
    vy: f32,
    prev_speed: f32,
}

struct Sim {
    cells: Vec<Cell>,
    order: Vec<usize>,
    rng: u64,
    g: f32,
    softening: f32,
    speed_cap: f32,
    start_pop: usize,
    conway_every: usize,
    pop_band: f32,
    tick_count: usize,
    prev_live: Vec<bool>,
    wrap: bool,   // toroidal wrapping (false = hard walls)
    steer: bool,  // counter-rotate velocity to compensate discrete-move angular error
    conway_births: usize,  // cumulative Conway births
    conway_deaths: usize,  // cumulative Conway deaths
}

// Original state captured at tick=0 for epilogue convergence
struct OriginalState {
    positions: std::collections::HashSet<(usize, usize)>,
    // Map from grid position → original velocity
    velocities: std::collections::HashMap<(usize, usize), (f32, f32)>,
    count: usize,
}

// ── Barnes-Hut quadtree for O(n log n) gravity ────────────────────────────
const BH_THETA: f32 = 0.5; // opening-angle criterion: width/dist < theta → use point-mass

#[derive(Clone)]
struct QNode {
    x0: f32, y0: f32, x1: f32, y1: f32, // bounding box
    com_x: f32, com_y: f32,              // centre of mass
    mass: f32,                           // particle count in subtree
    body: i32,    // ≥0: leaf with one particle; -1: internal; -2: empty
    ch: [i32; 4], // arena indices of children (q=0..3), -1 = absent
}

impl QNode {
    fn empty(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        QNode { x0, y0, x1, y1, com_x: 0.0, com_y: 0.0, mass: 0.0, body: -2, ch: [-1; 4] }
    }
    #[inline] fn mid_x(&self) -> f32 { (self.x0 + self.x1) * 0.5 }
    #[inline] fn mid_y(&self) -> f32 { (self.y0 + self.y1) * 0.5 }
    #[inline] fn width(&self) -> f32 { (self.x1 - self.x0).max(self.y1 - self.y0) }
    #[inline] fn quadrant(&self, x: f32, y: f32) -> usize {
        (if x >= self.mid_x() { 1 } else { 0 }) | (if y >= self.mid_y() { 2 } else { 0 })
    }
    fn child_box(&self, q: usize) -> (f32, f32, f32, f32) {
        let (mx, my) = (self.mid_x(), self.mid_y());
        match q {
            0 => (self.x0, self.y0, mx,       my      ),
            1 => (mx,       self.y0, self.x1, my      ),
            2 => (self.x0, my,       mx,       self.y1),
            _ => (mx,       my,       self.x1, self.y1),
        }
    }
}

fn qt_insert(nodes: &mut Vec<QNode>, idx: usize, body: usize, px: f32, py: f32, depth: u32) {
    if depth > 64 { return; } // safety: coincident particles
    match nodes[idx].body {
        -2 => {
            // Empty → leaf
            nodes[idx].body   = body as i32;
            nodes[idx].com_x  = px;
            nodes[idx].com_y  = py;
            nodes[idx].mass   = 1.0;
        }
        -1 => {
            // Internal: update COM, recurse into child
            let q = nodes[idx].quadrant(px, py);
            let child_idx = if nodes[idx].ch[q] < 0 {
                let (cx0, cy0, cx1, cy1) = nodes[idx].child_box(q);
                let c = nodes.len() as i32;
                nodes.push(QNode::empty(cx0, cy0, cx1, cy1));
                nodes[idx].ch[q] = c;
                c as usize
            } else {
                nodes[idx].ch[q] as usize
            };
            let m = nodes[idx].mass;
            nodes[idx].com_x = (nodes[idx].com_x * m + px) / (m + 1.0);
            nodes[idx].com_y = (nodes[idx].com_y * m + py) / (m + 1.0);
            nodes[idx].mass  += 1.0;
            qt_insert(nodes, child_idx, body, px, py, depth + 1);
        }
        old_body => {
            // Leaf → split: re-insert old particle, insert new
            let old_px = nodes[idx].com_x;
            let old_py = nodes[idx].com_y;
            let old_body = old_body as usize;
            // Convert to internal with 2-particle COM
            nodes[idx].body  = -1;
            nodes[idx].com_x = (old_px + px) * 0.5;
            nodes[idx].com_y = (old_py + py) * 0.5;
            nodes[idx].mass  = 2.0;
            // Re-insert old particle
            let q_old = nodes[idx].quadrant(old_px, old_py);
            let (cx0, cy0, cx1, cy1) = nodes[idx].child_box(q_old);
            let c_old = nodes.len() as i32;
            nodes.push(QNode::empty(cx0, cy0, cx1, cy1));
            nodes[idx].ch[q_old] = c_old;
            qt_insert(nodes, c_old as usize, old_body, old_px, old_py, depth + 1);
            // Insert new particle
            let q_new = nodes[idx].quadrant(px, py);
            let child_new = if nodes[idx].ch[q_new] < 0 {
                let (cx0, cy0, cx1, cy1) = nodes[idx].child_box(q_new);
                let c = nodes.len() as i32;
                nodes.push(QNode::empty(cx0, cy0, cx1, cy1));
                nodes[idx].ch[q_new] = c;
                c as usize
            } else {
                nodes[idx].ch[q_new] as usize
            };
            qt_insert(nodes, child_new, body, px, py, depth + 1);
        }
    }
}

#[inline]
fn min_image(d: f32, dim: f32) -> f32 {
    if d > dim * 0.5 { d - dim } else if d < -dim * 0.5 { d + dim } else { d }
}

fn qt_force(nodes: &[QNode], node_idx: usize, body: usize,
            px: f32, py: f32, g: f32, softening: f32, wrap: bool) -> (f32, f32) {
    let node = &nodes[node_idx];
    if node.body == -2 { return (0.0, 0.0); } // empty node
    let raw_dx = node.com_x - px;
    let raw_dy = node.com_y - py;
    let dx = if wrap { min_image(raw_dx, W as f32) } else { raw_dx };
    let dy = if wrap { min_image(raw_dy, H as f32) } else { raw_dy };
    // Leaf: exact pairwise force (skip self)
    if node.body >= 0 {
        if node.body as usize == body { return (0.0, 0.0); }
        let r2 = dx*dx + dy*dy + softening*softening;
        let r  = r2.sqrt();
        let f  = g * node.mass / r2;
        return (f * dx / r, f * dy / r);
    }
    // Internal: Barnes-Hut criterion uses actual (un-softened) distance
    let r2_actual = dx*dx + dy*dy;
    let d = r2_actual.sqrt();
    if d > 0.0 && node.width() / d < BH_THETA {
        // Far enough: treat as single point mass
        let r2 = r2_actual + softening*softening;
        let r  = r2.sqrt();
        let f  = g * node.mass / r2;
        return (f * dx / r, f * dy / r);
    }
    // Too close or at same position: recurse into children
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;
    for &ch in &node.ch {
        if ch >= 0 {
            let (cfx, cfy) = qt_force(nodes, ch as usize, body, px, py, g, softening, wrap);
            fx += cfx;
            fy += cfy;
        }
    }
    (fx, fy)
}
// ──────────────────────────────────────────────────────────────────────────

impl Sim {
    fn new(rng_seed: u64, g: f32, softening: f32, speed_cap: f32, conway_every: usize, pop_band: f32,
           clumps: &[(f32, f32, f32, f32, f32, usize)], seed_density_inv: usize,
           wrap: bool, steer: bool) -> Self {
        let mut rng = rng_seed;
        let mut cells = Vec::new();

        for &(cx, cy, r, ivx, ivy, _count) in clumps {
            let ri = r.ceil() as i32;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    if (dx as f32).powi(2) + (dy as f32).powi(2) > r * r { continue; }
                    let x = (cx + dx as f32).rem_euclid(W as f32);
                    let y = (cy + dy as f32).rem_euclid(H as f32);
                    let xi = x as usize;
                    let yi = y as usize;
                    // Checkerboard 50%, then randomly discard half ×3 → ~6.25% density
                    if xi % 2 != 0 || yi % 2 != 0 { continue; }
                    if xoru64(&mut rng) % 2 != 0 { continue; }
                    if xoru64(&mut rng) % 2 != 0 { continue; }
                    if xoru64(&mut rng) % 2 != 0 { continue; }
                    if cells.iter().any(|c: &Cell| c.x == xi && c.y == yi) {
                        continue;
                    }
                    cells.push(Cell { x: xi, y: yi, vx: ivx, vy: ivy, prev_speed: 0.0 });
                }
            }
        }
        shuffle_vec(&mut cells, &mut rng);

        // Seed 1/seed_density_inv of empty cells as zero-momentum live cells (0 = none)
        let mut occupied = vec![false; W * H];
        for c in &cells {
            occupied[c.y * W + c.x] = true;
        }
        let seed_count = if seed_density_inv > 0 {
            occupied.iter().filter(|&&v| !v).count() / seed_density_inv
        } else { 0 };
        let mut seeded = 0;
        for _ in 0..W * H * 4 {
            if seeded >= seed_count { break; }
            let xi = (xoru64(&mut rng) as usize) % W;
            let yi = (xoru64(&mut rng) as usize) % H;
            let idx = yi * W + xi;
            if !occupied[idx] {
                // Small random initial velocity: speed ~ U[0, 1% of speed_cap], random direction
                let spd   = xorf32(&mut rng) * speed_cap * 0.01;
                let angle = xorf32(&mut rng) * 2.0 * std::f32::consts::PI;
                let vx    = angle.cos() * spd;
                let vy    = angle.sin() * spd;
                cells.push(Cell { x: xi, y: yi, vx, vy, prev_speed: 0.0 });
                occupied[idx] = true;
                seeded += 1;
            }
        }
        shuffle_vec(&mut cells, &mut rng);

        let n = cells.len();
        let target_pop = W * H / 32;
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap, start_pop: target_pop,
              conway_every, pop_band, tick_count: 0, prev_live: vec![false; W * H], wrap, steer,
              conway_births: 0, conway_deaths: 0 }
    }

    // ── Checkpoint save/load ───────────────────────────────────────────────
    fn save_checkpoint(&self, canvas: &[f32], chunk_index: usize, path: &str) {
        let mut buf: Vec<u8> = Vec::new();
        // header
        buf.extend_from_slice(&(self.cells.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.rng.to_le_bytes());
        buf.extend_from_slice(&(self.tick_count as u64).to_le_bytes());
        buf.extend_from_slice(&(chunk_index as u64).to_le_bytes());
        // cells
        for c in &self.cells {
            buf.extend_from_slice(&(c.x as u32).to_le_bytes());
            buf.extend_from_slice(&(c.y as u32).to_le_bytes());
            buf.extend_from_slice(&c.vx.to_le_bytes());
            buf.extend_from_slice(&c.vy.to_le_bytes());
            buf.extend_from_slice(&c.prev_speed.to_le_bytes());
        }
        // prev_live (packed as u8 per bool for simplicity)
        for &b in &self.prev_live {
            buf.push(b as u8);
        }
        // canvas (f32 per channel)
        for &v in canvas {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        fs::write(path, &buf).unwrap();
    }

    fn load_checkpoint(path: &str, g: f32, softening: f32, speed_cap: f32,
                       conway_every: usize, pop_band: f32, _seed_density_inv: usize,
                       wrap: bool, steer: bool)
        -> Option<(Self, Vec<f32>, usize)>
    {
        let buf = fs::read(path).ok()?;
        let mut pos = 0;
        macro_rules! read_u64 {
            () => {{ let v = u64::from_le_bytes(buf[pos..pos+8].try_into().unwrap()); pos += 8; v }};
        }
        macro_rules! read_f32 {
            () => {{ let v = f32::from_le_bytes(buf[pos..pos+4].try_into().unwrap()); pos += 4; v }};
        }
        let n_cells = read_u64!() as usize;
        let rng     = read_u64!();
        let tick_count = read_u64!() as usize;
        let chunk_index = read_u64!() as usize;

        macro_rules! read_u32 {
            () => {{ let v = u32::from_le_bytes(buf[pos..pos+4].try_into().unwrap()); pos += 4; v }};
        }
        let mut cells = Vec::with_capacity(n_cells);
        for _ in 0..n_cells {
            let x  = read_u32!() as usize;
            let y  = read_u32!() as usize;
            let vx = read_f32!();
            let vy = read_f32!();
            let ps = read_f32!();
            cells.push(Cell { x, y, vx, vy, prev_speed: ps });
        }

        let mut prev_live = vec![false; W * H];
        for b in prev_live.iter_mut() {
            *b = buf[pos] != 0; pos += 1;
        }

        let mut canvas = vec![0.0f32; W * H * 3];
        for v in canvas.iter_mut() {
            *v = read_f32!();
        }

        // Rebuild prev_live from cell positions so first painted frame does correct 50% snap
        // (if we used the saved prev_live, a SIGTERM mid-tick could leave it stale)
        let mut prev_live_rebuilt = vec![false; W * H];
        for c in &cells {
            prev_live_rebuilt[c.y * W + c.x] = true;
        }
        let order = (0..cells.len()).collect();
        let target_pop = W * H / 32;
        let sim = Sim { cells, order, rng, g, softening, speed_cap,
                        start_pop: target_pop, conway_every, pop_band,
                        tick_count, prev_live: prev_live_rebuilt, wrap, steer };
        Some((sim, canvas, chunk_index))
    }

    // ── Original-state save/load (for correct epilogue target) ───────────
    // Saved once at fresh-start tick=0; loaded on checkpoint resume so the
    // epilogue always converges toward the very first frame of the simulation.
    fn save_orig_state(orig: &OriginalState, path: &str) {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&(orig.count as u64).to_le_bytes());
        // Save as sorted list of (x, y, vx, vy) — ordered for determinism
        let mut entries: Vec<((usize,usize),(f32,f32))> = orig.velocities.iter()
            .map(|(&pos, &vel)| (pos, vel)).collect();
        entries.sort_unstable_by_key(|&((x,y),_)| (y,x));
        for ((x,y),(vx,vy)) in &entries {
            buf.extend_from_slice(&(*x as u64).to_le_bytes());
            buf.extend_from_slice(&(*y as u64).to_le_bytes());
            buf.extend_from_slice(&vx.to_le_bytes());
            buf.extend_from_slice(&vy.to_le_bytes());
        }
        fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        fs::write(path, &buf).unwrap();
    }

    fn load_orig_state(path: &str) -> Option<OriginalState> {
        let buf = fs::read(path).ok()?;
        let mut pos = 0;
        macro_rules! read_u64 { () => {{ let v = u64::from_le_bytes(buf[pos..pos+8].try_into().ok()?); pos += 8; v }}; }
        macro_rules! read_f32 { () => {{ let v = f32::from_le_bytes(buf[pos..pos+4].try_into().ok()?); pos += 4; v }}; }
        let count = read_u64!() as usize;
        let mut positions = std::collections::HashSet::with_capacity(count);
        let mut velocities = std::collections::HashMap::with_capacity(count);
        for _ in 0..count {
            let x  = read_u64!() as usize;
            let y  = read_u64!() as usize;
            let vx = read_f32!();
            let vy = read_f32!();
            positions.insert((x, y));
            velocities.insert((x, y), (vx, vy));
        }
        Some(OriginalState { positions, velocities, count })
    }

    // ── Conway step ────────────────────────────────────────────────────────
    fn conway_step(&mut self) {
        shuffle_vec(&mut self.cells, &mut self.rng);

        let n = self.cells.len();
        let pop_min = self.start_pop.saturating_sub(self.pop_band as usize);
        let pop_max = self.start_pop + self.pop_band as usize;

        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y as usize % H * W + c.x as usize % W] = i;
        }

        let neighbour_offsets: [(i32, i32); 8] = [
            (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
        ];
        let wrap = self.wrap;
        let live_neighbours = |gy: usize, gx: usize| -> Vec<usize> {
            neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ry = gy as i32 + dy;
                let rx = gx as i32 + dx;
                let (ny, nx) = if wrap {
                    (ry.rem_euclid(H as i32) as usize, rx.rem_euclid(W as i32) as usize)
                } else {
                    if ry < 0 || ry >= H as i32 || rx < 0 || rx >= W as i32 { return None; }
                    (ry as usize, rx as usize)
                };
                let idx = grid[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect()
        };

        let mut desired_births: Vec<(usize, usize, Vec<usize>)> = Vec::new();
        let mut desired_deaths: Vec<usize> = Vec::new();

        for (i, c) in self.cells.iter().enumerate() {
            let gx = c.x as usize % W;
            let gy = c.y as usize % H;
            let nbrs = live_neighbours(gy, gx);
            let count = nbrs.len();
            if count != 2 && count != 3 {
                if !nbrs.is_empty() { desired_deaths.push(i); }
            }
        }

        let mut candidates = std::collections::HashSet::new();
        for c in &self.cells {
            let gx = c.x as usize % W;
            let gy = c.y as usize % H;
            for &(dy, dx) in &neighbour_offsets {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                if grid[ny * W + nx] == usize::MAX { candidates.insert((ny, nx)); }
            }
        }
        for (gy, gx) in candidates {
            let nbrs = live_neighbours(gy, gx);
            if nbrs.len() == 3 { desired_births.push((gy, gx, nbrs)); }
        }

        shuffle_vec(&mut desired_deaths, &mut self.rng);
        // Deaths: uniform random selection (shuffled above)
        // Births: weighted by neighbour speed — handled below after grid2 is built

        // Rate-limit: max births/deaths per Conway call, independent of pop_band.
        // With conway_every=FPS/4 (4 calls/sec) and rate_limit=4: up to 16 births+deaths/sec.
        let rate_limit = 4_usize;
        let max_births = pop_max.saturating_sub(n).min(rate_limit);
        let max_deaths = n.saturating_sub(pop_min).min(rate_limit);
        // desired_births NOT truncated here — weighted selection happens post-deaths
        desired_deaths.truncate(max_deaths);

        let mut dying: std::collections::HashSet<usize> = desired_deaths.iter().cloned().collect();

        for &di in &dying {
            let (dvx, dvy) = (self.cells[di].vx, self.cells[di].vy);
            let gx = self.cells[di].x as usize % W;
            let gy = self.cells[di].y as usize % H;
            let receivers: Vec<usize> = live_neighbours(gy, gx).into_iter()
                .filter(|&ni| !dying.contains(&ni)).collect();
            if !receivers.is_empty() {
                let share = 1.0 / receivers.len() as f32;
                for &ri in &receivers {
                    self.cells[ri].vx += dvx * share;
                    self.cells[ri].vy += dvy * share;
                }
            }
        }

        let mut death_indices: Vec<usize> = dying.drain().collect();
        death_indices.sort_unstable_by(|a, b| b.cmp(a));
        for i in death_indices { self.cells.swap_remove(i); }

        let mut grid2 = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid2[c.y as usize % H * W + c.x as usize % W] = i;
        }

        // Weighted birth selection: candidates with faster-moving neighbours are
        // proportionally more likely to be born. Uses Efraimidis-Spirakis reservoir
        // sampling: key = u^(1/w), sort descending, take top max_births.
        //
        // weight = sum of live-neighbour speeds (post-deaths) + BIRTH_SOFT
        // BIRTH_SOFT ensures every valid candidate has a nonzero base probability.
        const BIRTH_SOFT: f32 = 0.005; // ~1/10 of speed_cap; baseline birth weight

        let mut birth_keys: Vec<(f32, usize)> = desired_births.iter()
            .enumerate()
            .filter_map(|(i, (gy, gx, _))| {
                if grid2[gy * W + gx] != usize::MAX { return None; } // already occupied
                let spd_sum: f32 = neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                    let (ny, nx) = resolve_nbr(*gy, *gx, dy, dx)?;
                    let idx = grid2[ny * W + nx];
                    if idx != usize::MAX {
                        let c = &self.cells[idx];
                        Some((c.vx * c.vx + c.vy * c.vy).sqrt())
                    } else { None }
                }).sum();
                let w = spd_sum + BIRTH_SOFT;
                let u = xorf32(&mut self.rng).max(f32::EPSILON); // avoid u=0
                Some((u.powf(1.0 / w), i))
            })
            .collect();

        // Sort descending by key — highest key = most likely to be selected
        birth_keys.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, bi) in birth_keys.into_iter().take(max_births) {
            let (gy, gx, _) = desired_births[bi];
            if grid2[gy * W + gx] != usize::MAX { continue; } // double-check: may have been filled
            let live_nbrs: Vec<usize> = neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                let idx = grid2[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect();
            if live_nbrs.is_empty() { continue; }
            let n_nbrs = live_nbrs.len() as f32;
            let vx = live_nbrs.iter().map(|&i| self.cells[i].vx).sum::<f32>() / n_nbrs;
            let vy = live_nbrs.iter().map(|&i| self.cells[i].vy).sum::<f32>() / n_nbrs;
            let birth_spd = (vx * vx + vy * vy).sqrt();
            let new_idx = self.cells.len();
            self.cells.push(Cell { x: gx as f32 + 0.5, y: gy as f32 + 0.5, vx, vy, prev_speed: birth_spd });
            grid2[gy * W + gx] = new_idx;
        }

        self.order = (0..self.cells.len()).collect();
    }

    // ── Gravity step (Barnes-Hut O(n log n)) ──────────────────────────────
    fn gravity_step(&mut self) {
        let n = self.cells.len();

        // Build quadtree over the toroidal domain (use cell centres for continuous physics)
        let mut nodes: Vec<QNode> = Vec::with_capacity(n * 8);
        nodes.push(QNode::empty(0.0, 0.0, W as f32, H as f32));
        for i in 0..n {
            let (px, py) = (self.cells[i].x as f32 + 0.5, self.cells[i].y as f32 + 0.5);
            qt_insert(&mut nodes, 0, i, px, py, 0);
        }

        // Compute gravitational force on each particle via tree traversal
        for i in 0..n {
            let (px, py) = (self.cells[i].x as f32 + 0.5, self.cells[i].y as f32 + 0.5);
            let (gfx, gfy) = qt_force(&nodes, 0, i, px, py, self.g, self.softening);
            self.cells[i].vx += gfx;
            self.cells[i].vy += gfy;
        }

        for c in &mut self.cells {
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            let effective_cap = c.prev_speed.max(self.speed_cap);
            if spd > effective_cap {
                c.vx = c.vx / spd * effective_cap;
                c.vy = c.vy / spd * effective_cap;
            }
            let hard_ceil = self.speed_cap * 2.0;
            c.prev_speed = c.prev_speed.min(spd).max(self.speed_cap).min(hard_ceil);
        }

        // Momentum damping: nudge system average velocity toward zero by 1/128 per tick.
        // Prevents the centre-of-mass from drifting due to simulation asymmetries.
        if !self.cells.is_empty() {
            let n = self.cells.len() as f32;
            let avg_vx = self.cells.iter().map(|c| c.vx).sum::<f32>() / n;
            let avg_vy = self.cells.iter().map(|c| c.vy).sum::<f32>() / n;
            let damp = 1.0 / 128.0;
            for c in &mut self.cells {
                c.vx -= avg_vx * damp;
                c.vy -= avg_vy * damp;
            }
        }

        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y * W + c.x] = i;
        }

        // Save pre-move positions for steer correction
        let old_pos: Vec<(usize, usize)> = self.cells.iter().map(|c| (c.x, c.y)).collect();

        // Target integer position.
        // Wrap mode: toroidal (rem_euclid). No-wrap mode: stay put if target is out of bounds.
        // IMPORTANT: use round(), not truncation (cast). Truncation biases movement toward
        // -x/-y: vx∈(-1,0) always moves left, vx∈(0,1) never moves right → top-left drift.
        let target_pos: Vec<(usize, usize)> = self.cells.iter().map(|c| {
            let raw_x = c.x as f32 + c.vx;
            let raw_y = c.y as f32 + c.vy;
            if self.wrap {
                let tx = (raw_x.round() as i32).rem_euclid(W as i32) as usize;
                let ty = (raw_y.round() as i32).rem_euclid(H as i32) as usize;
                (tx, ty)
            } else {
                // Out of bounds → don't move (same rule as occupied cell)
                let rx = raw_x.round();
                let ry = raw_y.round();
                if rx < 0.0 || rx >= W as f32 || ry < 0.0 || ry >= H as f32 {
                    (c.x, c.y)
                } else {
                    (rx as usize, ry as usize)
                }
            }
        }).collect();

        let n = self.order.len();
        for i in (1..n).rev() {
            let j = (xoru64(&mut self.rng) as usize) % (i + 1);
            self.order.swap(i, j);
        }

        let mut reservation: Vec<usize> = vec![usize::MAX; W * H];
        let mut moved = vec![false; n];

        for &idx in &self.order {
            let (tx, ty) = target_pos[idx];
            let old_x = self.cells[idx].x;
            let old_y = self.cells[idx].y;

            if tx == old_x && ty == old_y {
                moved[idx] = true;
                continue;
            }

            if grid[ty * W + tx] == usize::MAX {
                grid[old_y * W + old_x] = usize::MAX;
                grid[ty * W + tx] = idx;
                self.cells[idx].x = tx;
                self.cells[idx].y = ty;
                moved[idx] = true;

                let mut freed = old_y * W + old_x;
                loop {
                    let waiter = reservation[freed];
                    if waiter == usize::MAX { break; }
                    reservation[freed] = usize::MAX;
                    let (wtx, wty) = target_pos[waiter];
                    let wox = self.cells[waiter].x;
                    let woy = self.cells[waiter].y;
                    grid[woy * W + wox] = usize::MAX;
                    grid[wty * W + wtx] = waiter;
                    self.cells[waiter].x = wtx;
                    self.cells[waiter].y = wty;
                    moved[waiter] = true;
                    freed = woy * W + wox;
                }
            } else {
                let key = ty * W + tx;
                if reservation[key] == usize::MAX { reservation[key] = idx; }
            }
        }

        // Steer correction: counter-rotate velocity by the angular error introduced by
        // discrete grid movement. If the grid forced a cell 20° clockwise of its intended
        // direction, rotate the velocity 20° counter-clockwise to compensate.
        if self.steer {
            for idx in 0..n {
                if !moved[idx] { continue; }
                let (ox, oy) = old_pos[idx];
                let (nx, ny) = (self.cells[idx].x, self.cells[idx].y);
                if nx == ox && ny == oy { continue; } // stayed in same square, no error
                // Actual displacement (with min-image for wrap, direct for no-wrap)
                let adx = if self.wrap { min_image(nx as f32 - ox as f32, W as f32) }
                           else { nx as f32 - ox as f32 };
                let ady = if self.wrap { min_image(ny as f32 - oy as f32, H as f32) }
                           else { ny as f32 - oy as f32 };
                let spd = (self.cells[idx].vx.powi(2) + self.cells[idx].vy.powi(2)).sqrt();
                if spd == 0.0 { continue; }
                let intended = self.cells[idx].vy.atan2(self.cells[idx].vx);
                let actual   = ady.atan2(adx);
                let error    = actual - intended; // how much the grid rotated us
                let corrected = intended - error; // rotate back by the same amount
                self.cells[idx].vx = corrected.cos() * spd;
                self.cells[idx].vy = corrected.sin() * spd;
            }
        }
    }

    fn tick(&mut self) {
        if self.conway_every > 0 && self.tick_count % self.conway_every == 0 {
            self.conway_step();
        }
        self.gravity_step();
        self.tick_count += 1;
    }

    // Epilogue tick: ramp Conway down, ramp nudges up, converge to original state.
    // epilogue_tick: 0-based tick within epilogue phase.
    // Returns true when converged.
    fn epilogue_tick(&mut self, orig: &OriginalState, epilogue_tick: usize) -> bool {
        const RAMP_TICKS: usize = 32; // fully ramped by frame 32
        let t = (epilogue_tick as f32 / RAMP_TICKS as f32).min(1.0);

        // Conway deaths only (no births) with ramping-down rate — clears non-original cells.
        // Births are handled exclusively by the revive nudge, ensuring only original positions get filled.
        let conway_max = (8.0 * (1.0 - t)).floor() as usize;
        if conway_max > 0 {
            self.epilogue_conway_deaths_only(conway_max);
        }

        // Gravity still runs (frozen cells handled by not moving them)
        self.gravity_step_epilogue(orig, 1.0);

        // Nudges: ramp up
        let kill_chance   = 0.75 * t;
        let revive_chance = 0.375 * t;

        // Build current live set
        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y as usize % H * W + c.x as usize % W] = i;
        }

        // Per-cell nudges: every non-original live cell has kill_chance of dying,
        // every dead original cell has revive_chance of being born.
        // Collect indices to kill (high to low for swap_remove stability)
        let mut to_kill: Vec<usize> = self.cells.iter().enumerate()
            .filter(|(_, c)| !orig.positions.contains(&(c.x as usize % W, c.y as usize % H)))
            .filter(|_| xorf32(&mut self.rng) < kill_chance)
            .map(|(i, _)| i)
            .collect();
        to_kill.sort_unstable_by(|a, b| b.cmp(a));
        for i in to_kill { self.cells.swap_remove(i); }

        // Rebuild grid after kills
        let mut grid2 = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid2[c.y as usize % H * W + c.x as usize % W] = i;
        }

        // Every dead original cell has revive_chance of being born
        for &(ox, oy) in &orig.positions {
            if grid2[oy * W + ox] == usize::MAX && xorf32(&mut self.rng) < revive_chance {
                self.cells.push(Cell { x: ox as f32 + 0.5, y: oy as f32 + 0.5,
                                       vx: 0.0, vy: 0.0, prev_speed: 0.0 });
            }
        }

        // Lerp velocities of live-original cells 3.125% closer to their original velocity each tick (4× slower)
        for c in &mut self.cells {
            let pos = (c.x as usize % W, c.y as usize % H);
            if let Some(&(tvx, tvy)) = orig.velocities.get(&pos) {
                c.vx += (tvx - c.vx) * 0.03125; // 3.125%/tick = 12.5%/tick ÷ 4
                c.vy += (tvy - c.vy) * 0.03125;
            }
        }

        self.tick_count += 1;
        self.order = (0..self.cells.len()).collect();

        // Check convergence: positions match — velocity phase handled separately
        if self.cells.len() == orig.count {
            let live: std::collections::HashSet<(usize,usize)> = self.cells.iter()
                .map(|c| (c.x as usize % W, c.y as usize % H))
                .collect();
            if live == orig.positions {
                return true;
            }
        }
        false
    }

    // Velocity convergence phase: called for up to 64 ticks once positions have converged.
    // Ramps gravity AND life (Conway deaths + nudge kill/revive) to 0 over the first 32 ticks.
    // Guarantees velocity error never increases: snapshots pre-gravity error, clamps any
    // growth after gravity runs, then lerps toward target.
    // conv_t: the RAMP_TICKS t-value at the tick positions converged (used to compute
    //         residual Conway/nudge rates at entry so the ramp starts from that level).
    fn epilogue_vel_tick(&mut self, orig: &OriginalState, vel_phase_tick: usize, conv_t: f32) {
        let g_scale    = (1.0 - vel_phase_tick as f32 / 32.0).max(0.0);
        let life_scale = (1.0 - vel_phase_tick as f32 / 32.0).max(0.0);

        // Ramp Conway deaths down to zero over 32 frames (residual from position phase).
        let conway_max = (8.0 * (1.0 - conv_t) * life_scale).floor() as usize;
        if conway_max > 0 {
            self.epilogue_conway_deaths_only(conway_max);
        }

        // Kill/revive nudges: ramp from convergence strength up to full over 64 frames.
        // These stay active to correct any cells displaced by residual gravity.
        let vel_scale    = conv_t + (1.0 - conv_t) * (vel_phase_tick as f32 / 64.0).min(1.0);
        let kill_chance   = 0.75  * vel_scale;
        let revive_chance = 0.375 * vel_scale;
        {
            let mut grid = vec![usize::MAX; W * H];
            for (i, c) in self.cells.iter().enumerate() {
                grid[c.y as usize % H * W + c.x as usize % W] = i;
            }
            let mut to_kill: Vec<usize> = self.cells.iter().enumerate()
                .filter(|(_, c)| !orig.positions.contains(&(c.x as usize % W, c.y as usize % H)))
                .filter(|_| xorf32(&mut self.rng) < kill_chance)
                .map(|(i, _)| i)
                .collect();
            to_kill.sort_unstable_by(|a, b| b.cmp(a));
            for i in to_kill { self.cells.swap_remove(i); }

            let mut grid2 = vec![usize::MAX; W * H];
            for (i, c) in self.cells.iter().enumerate() {
                grid2[c.y as usize % H * W + c.x as usize % W] = i;
            }
            for &(ox, oy) in &orig.positions {
                if grid2[oy * W + ox] == usize::MAX && xorf32(&mut self.rng) < revive_chance {
                    self.cells.push(Cell { x: ox as f32 + 0.5, y: oy as f32 + 0.5,
                                           vx: 0.0, vy: 0.0, prev_speed: 0.0 });
                }
            }
        }

        // Snapshot pre-gravity squared error for each cell
        let pre_err_sq: Vec<f32> = self.cells.iter().map(|c| {
            let pos = (c.x as usize % W, c.y as usize % H);
            let (tvx, tvy) = orig.velocities.get(&pos).copied().unwrap_or((0.0, 0.0));
            (c.vx - tvx).powi(2) + (c.vy - tvy).powi(2)
        }).collect();

        self.gravity_step_epilogue(orig, g_scale);

        // Clamp + lerp: error from target can only stay the same or shrink
        for (i, c) in self.cells.iter_mut().enumerate() {
            let pos = (c.x as usize % W, c.y as usize % H);
            let (tvx, tvy) = orig.velocities.get(&pos).copied().unwrap_or((0.0, 0.0));
            let dvx = c.vx - tvx;
            let dvy = c.vy - tvy;
            let post_err_sq = dvx * dvx + dvy * dvy;
            // Clamp: if gravity increased the error, project back to pre-gravity magnitude
            if post_err_sq > pre_err_sq[i] && pre_err_sq[i] > 0.0 {
                let scale = (pre_err_sq[i] / post_err_sq).sqrt();
                c.vx = tvx + dvx * scale;
                c.vy = tvy + dvy * scale;
            }
            // Lerp toward target velocity (3.125% per tick)
            c.vx += (tvx - c.vx) * 0.03125;
            c.vy += (tvy - c.vy) * 0.03125;
        }

        self.tick_count += 1;
        self.order = (0..self.cells.len()).collect();
    }

    fn epilogue_conway_deaths_only(&mut self, max_deaths: usize) {
        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y as usize % H * W + c.x as usize % W] = i;
        }
        let neighbour_offsets: [(i32, i32); 8] = [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)];
        let mut deaths: Vec<usize> = Vec::new();
        for (i, c) in self.cells.iter().enumerate() {
            let gx = c.x as usize % W; let gy = c.y as usize % H;
            let cnt = neighbour_offsets.iter().filter(|&&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                grid[ny * W + nx] != usize::MAX
            }).count();
            if cnt != 2 && cnt != 3 { deaths.push(i); }
        }
        shuffle_vec(&mut deaths, &mut self.rng);
        deaths.truncate(max_deaths);
        let dying: std::collections::HashSet<usize> = deaths.into_iter().collect();
        let mut di: Vec<usize> = dying.into_iter().collect();
        di.sort_unstable_by(|a, b| b.cmp(a));
        for i in di { self.cells.swap_remove(i); }
        self.order = (0..self.cells.len()).collect();
    }

    fn epilogue_conway_step(&mut self, max_per_component: usize, target_pop: usize) {
        // Simplified Conway step with fixed max births/deaths (no pop_band logic)
        shuffle_vec(&mut self.cells, &mut self.rng);
        let n = self.cells.len();
        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y as usize % H * W + c.x as usize % W] = i;
        }
        let neighbour_offsets: [(i32, i32); 8] = [
            (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
        ];
        let wrap = self.wrap;
        let live_neighbours = |gy: usize, gx: usize| -> Vec<usize> {
            neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ry = gy as i32 + dy;
                let rx = gx as i32 + dx;
                let (ny, nx) = if wrap {
                    (ry.rem_euclid(H as i32) as usize, rx.rem_euclid(W as i32) as usize)
                } else {
                    if ry < 0 || ry >= H as i32 || rx < 0 || rx >= W as i32 { return None; }
                    (ry as usize, rx as usize)
                };
                let idx = grid[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect()
        };
        let mut deaths: Vec<usize> = Vec::new();
        let mut births: Vec<(usize, usize, Vec<usize>)> = Vec::new();
        for (i, c) in self.cells.iter().enumerate() {
            let gx = c.x as usize % W; let gy = c.y as usize % H;
            let nbrs = live_neighbours(gy, gx);
            let cnt = nbrs.len();
            if cnt != 2 && cnt != 3 && !nbrs.is_empty() { deaths.push(i); }
        }
        let mut candidates = std::collections::HashSet::new();
        for c in &self.cells {
            let gx = c.x as usize % W; let gy = c.y as usize % H;
            for &(dy, dx) in &neighbour_offsets {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                if grid[ny * W + nx] == usize::MAX { candidates.insert((ny, nx)); }
            }
        }
        for (gy, gx) in candidates {
            let nbrs = live_neighbours(gy, gx);
            if nbrs.len() == 3 { births.push((gy, gx, nbrs)); }
        }
        shuffle_vec(&mut deaths, &mut self.rng);
        shuffle_vec(&mut births, &mut self.rng);
        let max_births = (target_pop.saturating_sub(n)).min(max_per_component);
        let max_deaths = (n.saturating_sub(target_pop)).min(max_per_component);
        deaths.truncate(max_deaths);
        births.truncate(max_births);
        let dying: std::collections::HashSet<usize> = deaths.iter().cloned().collect();
        let mut di: Vec<usize> = dying.iter().cloned().collect();
        di.sort_unstable_by(|a, b| b.cmp(a));
        for i in di { self.cells.swap_remove(i); }
        let mut grid2 = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid2[c.y as usize % H * W + c.x as usize % W] = i;
        }
        for (gy, gx, _) in births {
            if grid2[gy * W + gx] != usize::MAX { continue; }
            let live_nbrs: Vec<usize> = neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                let idx = grid2[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect();
            if live_nbrs.is_empty() { continue; }
            let n_nbrs = live_nbrs.len() as f32;
            let vx = live_nbrs.iter().map(|&i| self.cells[i].vx).sum::<f32>() / n_nbrs;
            let vy = live_nbrs.iter().map(|&i| self.cells[i].vy).sum::<f32>() / n_nbrs;
            let new_idx = self.cells.len();
            let spd = (vx*vx+vy*vy).sqrt();
            self.cells.push(Cell { x: gx as f32+0.5, y: gy as f32+0.5, vx, vy, prev_speed: spd });
            grid2[gy * W + gx] = new_idx;
        }
        self.order = (0..self.cells.len()).collect();
    }

    // Gravity step where original-position cells don't move (but still exert gravity)
    fn gravity_step_epilogue(&mut self, orig: &OriginalState) {
        let n = self.cells.len();
        // Barnes-Hut tree for epilogue gravity (original particles are "fixed" — no force applied)
        let mut nodes: Vec<QNode> = Vec::with_capacity(n * 8);
        nodes.push(QNode::empty(0.0, 0.0, W as f32, H as f32));
        for i in 0..n {
            let (px, py) = (self.cells[i].x, self.cells[i].y);
            qt_insert(&mut nodes, 0, i, px, py, 0);
        }
        for i in 0..n {
            let is_orig = orig.positions.contains(&(self.cells[i].x as usize % W, self.cells[i].y as usize % H));
            if is_orig { continue; } // original particles are fixed, skip force
            let (px, py) = (self.cells[i].x, self.cells[i].y);
            let (fx, fy) = qt_force(&nodes, 0, i, px, py, self.g, self.softening);
            self.cells[i].vx += fx;
            self.cells[i].vy += fy;
        }
        // Cap speeds, then only move non-original cells
        for c in &mut self.cells {
            let spd = (c.vx*c.vx+c.vy*c.vy).sqrt();
            let cap = c.prev_speed.max(self.speed_cap);
            if spd > cap { c.vx = c.vx/spd*cap; c.vy = c.vy/spd*cap; }
            let hard_ceil = self.speed_cap * 2.0;
            c.prev_speed = c.prev_speed.min(spd).max(self.speed_cap).min(hard_ceil);
        }
        // Move only non-original cells
        for c in &mut self.cells {
            let xi = c.x as usize % W; let yi = c.y as usize % H;
            if !orig.positions.contains(&(xi, yi)) {
                c.x = (c.x + c.vx).rem_euclid(W as f32);
                c.y = (c.y + c.vy).rem_euclid(H as f32);
            }
        }
        self.order = (0..self.cells.len()).collect();
    }

    fn paint_frame(&mut self, canvas: &mut Vec<f32>) {
        for py in 0..H {
            for px in 0..W {
                let i = (py * W + px) * 3;
                if self.prev_live[py * W + px] {
                    canvas[i]     *= 0.5;
                    canvas[i + 1] *= 0.5;
                    canvas[i + 2] *= 0.5;
                } else {
                    canvas[i]     *= 0.999767; // half fade rate vs 0.999534
                    canvas[i + 1] *= 0.999767;
                    canvas[i + 2] *= 0.999767;
                }
            }
        }
        self.prev_live.fill(false);
        for c in &self.cells {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            let (r, g, b) = velocity_color(c.vx, c.vy, self.speed_cap);
            let i = (yi * W + xi) * 3;
            canvas[i]     = r as f32;
            canvas[i + 1] = g as f32;
            canvas[i + 2] = b as f32;
            self.prev_live[yi * W + xi] = true;
        }
    }

    fn save_png(canvas: &[f32], path: &str) {
        let pixels: Vec<u8> = canvas.iter().map(|&v| v.clamp(0.0, 255.0) as u8).collect();
        let file = fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(BufWriter::new(file), W as u32, H as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }

    fn stats(&self) -> String {
        let n = self.cells.len() as f32;
        let avg_spd = self.cells.iter().map(|c| (c.vx*c.vx+c.vy*c.vy).sqrt()).sum::<f32>() / n;
        let max_spd = self.cells.iter().map(|c| (c.vx*c.vx+c.vy*c.vy).sqrt()).fold(0.0f32, f32::max);
        let cx = self.cells.iter().map(|c| c.x as f32).sum::<f32>() / n;
        let cy = self.cells.iter().map(|c| c.y as f32).sum::<f32>() / n;
        let hw = W as f32 / 2.0; let hh = H as f32 / 2.0;
        let spread = self.cells.iter().map(|c| {
            let mut dx = c.x as f32 - cx; let mut dy = c.y as f32 - cy;
            if dx > hw { dx -= W as f32; } if dx < -hw { dx += W as f32; }
            if dy > hh { dy -= H as f32; } if dy < -hh { dy += H as f32; }
            (dx*dx+dy*dy).sqrt()
        }).sum::<f32>() / n;
        format!("pop={} avg_spd={avg_spd:.3} max={max_spd:.3} spread={spread:.1} com=({cx:.1},{cy:.1})", self.cells.len())
    }
}

fn shuffle_vec<T>(v: &mut Vec<T>, rng: &mut u64) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (xoru64(rng) as usize) % (i + 1);
        v.swap(i, j);
    }
}

fn velocity_color(vx: f32, vy: f32, speed_cap: f32) -> (u8, u8, u8) {
    let spd = (vx * vx + vy * vy).sqrt();
    // speed_cap → 75% saturation; 100% requires exceeding speed_cap (≥ 4/3 × speed_cap)
    let sat = (spd * 0.75 / speed_cap).clamp(0.0, 1.0);
    let hue = (vy.atan2(vx) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
    let (r, g, b) = hsv_to_rgb(hue, sat, 1.0);
    let floor = 64u8;
    (r.max(floor), g.max(floor), b.max(floor))
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as u32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, t, p), 1 => (q, v, p), 2 => (p, v, t),
        3 => (p, q, v), 4 => (t, p, v), _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn xoru64(s: &mut u64) -> u64 {
    *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s
}

fn xorf32(s: &mut u64) -> f32 {
    (xoru64(s) & 0xFFFFFF) as f32 / 0xFFFFFF as f32
}

fn encode_chunk(frames_dir: &str, seg_path: &str, n_frames: usize) {
    // ffmpeg glob requires sorted files — they're zero-padded so glob order = numeric order
    let scale = format!("scale={}:{}:flags=neighbor", OUT_W, OUT_H);
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-framerate", &FPS.to_string(),
            "-pattern_type", "glob",
            "-i", &format!("{frames_dir}/*.png"),
            "-vf", &scale,
            "-c:v", "libx264",
            "-crf", &CRF.to_string(),
            "-pix_fmt", "yuv420p",
            seg_path,
        ])
        .status()
        .expect("ffmpeg failed");
    assert!(status.success(), "ffmpeg exited non-zero for {seg_path}");
    println!("  encoded {n_frames} frames → {seg_path}");
}

fn delete_frames(frames_dir: &str) {
    for entry in fs::read_dir(frames_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |e| e == "png") {
            fs::remove_file(path).unwrap();
        }
    }
}

fn concat_segments(segments_file: &str, output: &str) {
    let status = Command::new("ffmpeg")
        .args(["-y", "-f", "concat", "-safe", "0", "-i", segments_file, "-c", "copy", output])
        .status()
        .expect("ffmpeg concat failed");
    assert!(status.success(), "ffmpeg concat failed");
}

fn main() {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let parse_arg = |flag: &str| -> Option<String> {
        args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
    };
    let seconds: usize = parse_arg("--seconds")
        .and_then(|s| s.parse().ok())
        .expect("Usage: gravity --seconds <N> [--radius <r>] [--seed-density <1/N>] [--epilogue]");
    let do_epilogue = args.iter().any(|a| a == "--epilogue");
    let wrap  = !args.iter().any(|a| a == "--no-wrap");  // default: toroidal wrap
    let steer = args.iter().any(|a| a == "--steer");     // default: off
    // --radius: circle radius (0 = no blobs), default 4
    let rng_seed: u64 = parse_arg("--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(44);
    let blob_radius: f32 = parse_arg("--radius")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0); // no blobs by default
    // --seed-density: random cells as 1/N of empty cells (0 = none)
    // Default 128 = 1/128 of empty cells (half as dense again)
    let seed_density_inv: usize = parse_arg("--seed-density")
        .and_then(|s| s.parse().ok())
        .unwrap_or(128);

    let total_frames = seconds * FPS as usize;
    let n_chunks = (total_frames + CHUNK_FRAMES - 1) / CHUNK_FRAMES;

    println!("gravity: {}s × {}fps = {} frames, {} chunks of {} frames",
        seconds, FPS, total_frames, n_chunks, CHUNK_FRAMES);

    // Sim parameters
    let g: f32 = parse_arg("--gravity")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.000300_f32); // 4× stronger gravity
    let softening   = 1.5_f32;
    let speed_cap   = 0.093750_f32; // 2× previous cap
    let conway_every = FPS as usize / 4; // run Conway 4× per second → up to 4 births + 4 deaths/sec
    let pop_band    = 8.0_f32; // gap halved: min stays same, max comes halfway down

    // Four clockwise blobs — radius from --radius (0 = no blobs)
    let clumps_owned: Vec<(f32, f32, f32, f32, f32, usize)> = if blob_radius > 0.0 {
        // 6 blobs at random positions with random cardinal-ish directions, speed=0.010
        let mut rng2: u64 = 12345;
        let speed = 0.010_f32;
        (0..6).map(|_| {
            let x = (xoru64(&mut rng2) as usize % (W - 2 * blob_radius as usize - 2)) as f32 + blob_radius + 1.0;
            let y = (xoru64(&mut rng2) as usize % (H - 2 * blob_radius as usize - 2)) as f32 + blob_radius + 1.0;
            // random angle
            let angle = (xoru64(&mut rng2) as f32 / u64::MAX as f32) * 2.0 * std::f32::consts::PI;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;
            (x, y, blob_radius, vx, vy, 0)
        }).collect()
    } else {
        vec![]
    };
    let clumps: &[(f32, f32, f32, f32, f32, usize)] = &clumps_owned;

    let checkpoint_path = "state/checkpoint.bin";
    let segments_dir    = "segments";
    let frames_dir      = "frames/chunk";
    let segments_file   = "segments.txt";
    let shared_dir = std::env::var("GRAVITY_SHARED_DIR")
        .unwrap_or_else(|_| String::from("/Users/matte/.openclaw/workspace/shared/gravity"));
    fs::create_dir_all(&shared_dir).ok();
    let output_file = format!("{}/gravity_{}s_seed{}.mp4", shared_dir, seconds, rng_seed);

    fs::create_dir_all(segments_dir).unwrap();
    fs::create_dir_all(frames_dir).unwrap();
    fs::create_dir_all("state").unwrap();

    // Load checkpoint or init fresh
    let (mut sim, mut canvas, start_chunk) =
        Sim::load_checkpoint(checkpoint_path, g, softening, speed_cap, conway_every, pop_band, seed_density_inv, wrap, steer)
        .map(|(s, c, ci)| {
            println!("Resuming from checkpoint: chunk {}/{}", ci, n_chunks);
            (s, c, ci)
        })
        .unwrap_or_else(|| {
            println!("Fresh start (radius={blob_radius}, seed_density=1/{seed_density_inv})");
            let s = Sim::new(rng_seed, g, softening, speed_cap, conway_every, pop_band, clumps, seed_density_inv, wrap, steer);
            let c = vec![0.0f32; W * H * 3];
            (s, c, 0)
        });

    let orig_state_path = "state/orig_state.bin";

    // OriginalState = tick=0 layout. On fresh start: capture now and persist.
    // On checkpoint resume: load from disk so epilogue targets the actual first frame.
    let orig = if start_chunk == 0 {
        // Fresh start — this IS tick=0
        let o = OriginalState {
            positions:  sim.cells.iter().map(|c| (c.x as usize % W, c.y as usize % H)).collect(),
            velocities: sim.cells.iter().map(|c| ((c.x as usize % W, c.y as usize % H), (c.vx, c.vy))).collect(),
            count: sim.cells.len(),
        };
        Sim::save_orig_state(&o, orig_state_path);
        println!("Saved original state ({} cells) for epilogue target.", o.count);
        o
    } else {
        // Checkpoint resume — load the tick=0 state saved on fresh start
        match Sim::load_orig_state(orig_state_path) {
            Some(o) => { println!("Loaded original state ({} cells) for epilogue target.", o.count); o }
            None => {
                println!("WARNING: orig_state.bin not found — epilogue will target checkpoint state, not tick=0.");
                OriginalState {
                    positions:  sim.cells.iter().map(|c| (c.x as usize % W, c.y as usize % H)).collect(),
                    velocities: sim.cells.iter().map(|c| ((c.x as usize % W, c.y as usize % H), (c.vx, c.vy))).collect(),
                    count: sim.cells.len(),
                }
            }
        }
    };

    // Graceful shutdown: SIGINT/SIGTERM sets flag; loops check it and break,
    // then the normal final-concat path runs with whatever chunks are done.
    let keep_running = Arc::new(AtomicBool::new(true));
    let kr = keep_running.clone();
    ctrlc::set_handler(move || {
        if kr.load(Ordering::Relaxed) {
            println!("\n[signal] Caught — discarding current chunk, concatenating completed segments...");
            kr.store(false, Ordering::Relaxed);
        }
    }).expect("Error setting signal handler");

    // Open/append segments list
    let mut seg_list = fs::OpenOptions::new()
        .create(true).append(true)
        .open(segments_file).unwrap();

    'chunks: for chunk in start_chunk..n_chunks {
        let chunk_start_frame = chunk * CHUNK_FRAMES;
        let chunk_end_frame = ((chunk + 1) * CHUNK_FRAMES).min(total_frames);
        let this_chunk_frames = chunk_end_frame - chunk_start_frame;

        println!("\n[chunk {}/{n_chunks}] frames {}..{}", chunk+1, chunk_start_frame, chunk_end_frame);

        // Render frames for this chunk — check signal each frame
        for local_frame in 0..this_chunk_frames {
            if !keep_running.load(Ordering::Relaxed) {
                // Discard partial chunk and stop immediately
                println!("[signal] Discarding partial chunk {}, cleaning up {} frames...",
                    chunk + 1, local_frame);
                delete_frames(frames_dir);
                break 'chunks;
            }
            let global_frame = chunk_start_frame + local_frame;
            sim.paint_frame(&mut canvas);
            Sim::save_png(&canvas, &format!("{frames_dir}/f{global_frame:08}.png"));
            sim.tick();

            if local_frame % 480 == 0 {
                println!("  frame {}/{total_frames}  {}", global_frame, sim.stats());
            }
        }

        // Encode chunk
        let seg_path = format!("{segments_dir}/seg_{chunk_start_frame:08}.mp4");
        encode_chunk(frames_dir, &seg_path, this_chunk_frames);

        // Append to segments list
        writeln!(seg_list, "file '{seg_path}'").unwrap();
        seg_list.flush().unwrap();

        // Delete PNGs
        delete_frames(frames_dir);

        // Save checkpoint (next chunk index)
        sim.save_checkpoint(&canvas, chunk + 1, checkpoint_path);

        let pct = (chunk + 1) * 100 / n_chunks;
        println!("  chunk {}/{n_chunks} done ({pct}%)  pop={}", chunk+1, sim.cells.len());
    }

    // ── Epilogue phase ────────────────────────────────────────────────────
    if do_epilogue {
        println!("\n[epilogue] converging to original {} cells...", orig.count);
        const MAX_EPILOGUE_TICKS: usize = 240; // 4s hard cap
        let mut ep_tick = 0usize;
        let mut ep_frame = 0usize;
        let mut ep_chunk_frames: Vec<String> = Vec::new();
        let ep_seg_start = total_frames;
        let mut pos_converged = false;
        let mut vel_tick = 0usize;
        let mut conv_t = 0.0f32; // RAMP_TICKS t-value when positions converged

        loop {
            if !keep_running.load(Ordering::Relaxed) {
                // Flush any accumulated epilogue frames (already fully rendered), then stop
                if !ep_chunk_frames.is_empty() {
                    let seg_path = format!("{segments_dir}/seg_{:08}.mp4",
                        ep_seg_start + ep_frame - ep_chunk_frames.len());
                    encode_chunk(frames_dir, &seg_path, ep_chunk_frames.len());
                    writeln!(seg_list, "file '{seg_path}'").unwrap();
                    seg_list.flush().unwrap();
                    delete_frames(frames_dir);
                    ep_chunk_frames.clear();
                }
                println!("[signal] Stopping epilogue — concatenating completed segments.");
                break;
            }

            // Two-phase epilogue:
            // Phase 1 (position): kill/revive nudges until all cells at orig positions.
            // Phase 2 (velocity): up to 64 ticks, gravity ramps to 0 over first 32,
            //                     velocities clamped to never diverge from target.
            let done = if pos_converged {
                sim.epilogue_vel_tick(&orig, vel_tick, conv_t);
                vel_tick += 1;
                vel_tick >= 64
            } else {
                let pc = sim.epilogue_tick(&orig, ep_tick);
                if pc {
                    pos_converged = true;
                    conv_t = (ep_tick as f32 / 32.0_f32).min(1.0);
                    println!("  [epilogue] positions converged at tick {} ({:.1}s, t={:.2}) — velocity phase begins",
                        ep_tick, ep_tick as f32 / FPS as f32, conv_t);
                }
                false
            };

            // Ramp background fade: starts at normal rate, ramps to 0.5^0.25≈0.84/tick at full t
            let t = (ep_tick as f32 / 600.0_f32).min(1.0);
            let fade = 0.999767_f32.powf(1.0 - t) * 0.5_f32.powf(t * 0.25);
            for v in canvas.iter_mut() { *v *= fade; }
            sim.paint_frame(&mut canvas);
            let global_frame = total_frames + ep_frame;
            let path = format!("{frames_dir}/f{global_frame:08}.png");
            Sim::save_png(&canvas, &path);
            ep_chunk_frames.push(path);
            ep_frame += 1;
            ep_tick += 1;

            // Encode + flush every CHUNK_FRAMES frames
            if ep_chunk_frames.len() == CHUNK_FRAMES || done || ep_tick >= MAX_EPILOGUE_TICKS {
                if !ep_chunk_frames.is_empty() {
                    let seg_path = format!("{segments_dir}/seg_{:08}.mp4", ep_seg_start + ep_frame - ep_chunk_frames.len());
                    encode_chunk(frames_dir, &seg_path, ep_chunk_frames.len());
                    writeln!(seg_list, "file '{seg_path}'").unwrap();
                    seg_list.flush().unwrap();
                    delete_frames(frames_dir);
                    ep_chunk_frames.clear();
                }
            }

            if ep_tick % 120 == 0 {
                let live_orig = sim.cells.iter()
                    .filter(|c| orig.positions.contains(&(c.x as usize % W, c.y as usize % H)))
                    .count();
                let live_non_orig = sim.cells.len() - live_orig;
                let dead_orig = orig.count.saturating_sub(live_orig);
                println!("  epilogue t={:.2} pos_conv={} vel_tick={} pop={} live_orig={} non_orig={} dead_orig={}",
                    (ep_tick as f32 / 600.0).min(1.0), pos_converged, vel_tick,
                    sim.cells.len(), live_orig, live_non_orig, dead_orig);
            }
            if done { println!("  epilogue complete at tick {ep_tick} ({:.1}s)", ep_tick as f32 / FPS as f32); break; }
            if ep_tick >= MAX_EPILOGUE_TICKS { println!("  epilogue hit safety cap ({MAX_EPILOGUE_TICKS} ticks = 128s)"); break; }
        }
        println!("  epilogue: {ep_frame} frames appended");
    }

    // Final concat
    let total_segs = fs::read_to_string(segments_file).unwrap_or_default().lines().count();
    println!("\nConcatenating {total_segs} segments → {output_file}");
    concat_segments(segments_file, &output_file);

    let size = fs::metadata(&output_file).map(|m| m.len()).unwrap_or(0);
    println!("Done! {output_file} ({:.1} MB)", size as f64 / 1_048_576.0);

    // Clean up checkpoint on successful completion
    let _ = fs::remove_file(checkpoint_path);
    println!("Checkpoint removed.");
}
