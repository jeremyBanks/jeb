use std::fs;
use std::io::BufWriter;

const W: usize = 384;
const H: usize = 256;

struct Cell {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

struct Sim {
    cells: Vec<Cell>,
    order: Vec<usize>,
    rng: u64,
    g: f32,
    softening: f32,
    speed_cap: f32,
    start_pop: usize,
    conway_every: usize, // 0 = disabled, N = run Conway every N gravity ticks
    tick_count: usize,
    prev_live: Vec<bool>, // which pixels were live last frame (for fading)
}

impl Sim {
    fn new(rng_seed: u64, g: f32, softening: f32, speed_cap: f32, conway_every: usize,
           clumps: &[(f32, f32, f32, f32, f32, usize)]) -> Self {
        let mut rng = rng_seed;
        let mut cells = Vec::new();

        for &(cx, cy, r, ivx, ivy, _count) in clumps {
            // Fill every grid cell inside the circle
            let ri = r.ceil() as i32;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    if (dx as f32).powi(2) + (dy as f32).powi(2) > r * r { continue; }
                    let x = (cx + dx as f32).rem_euclid(W as f32);
                    let y = (cy + dy as f32).rem_euclid(H as f32);
                    let xi = x as usize;
                    let yi = y as usize;
                    if cells.iter().any(|c: &Cell| c.x as usize == xi && c.y as usize == yi) {
                        continue;
                    }
                    cells.push(Cell { x, y, vx: ivx, vy: ivy });
                }
            }
        }

        let n = cells.len();
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap, start_pop: n,
              conway_every, tick_count: 0, prev_live: vec![false; W * H] }
    }

    // ── Conway step (modified) ─────────────────────────────────────────────
    fn conway_step(&mut self) {
        let n = self.cells.len();
        let pop_min = self.start_pop / 2;
        let pop_max = self.start_pop * 2;

        // Build occupancy grid: cell index at each grid position (usize::MAX = empty)
        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            grid[yi * W + xi] = i;
        }

        // For each grid cell, count live neighbours and collect their indices
        let neighbour_offsets: [(i32, i32); 8] = [
            (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
        ];
        let live_neighbours = |gy: usize, gx: usize| -> Vec<usize> {
            neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                let idx = grid[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect()
        };

        // Determine desired births and deaths
        let mut desired_births: Vec<(usize, usize, Vec<usize>)> = Vec::new(); // (gy, gx, neighbour_indices)
        let mut desired_deaths: Vec<usize> = Vec::new(); // cell indices

        // Check all alive cells for death
        for (i, c) in self.cells.iter().enumerate() {
            let gx = c.x as usize % W;
            let gy = c.y as usize % H;
            let nbrs = live_neighbours(gy, gx);
            let count = nbrs.len();
            // Standard Conway: dies if not 2 or 3 neighbours
            if count != 2 && count != 3 {
                // Can only die if has at least one neighbour to receive velocity
                if !nbrs.is_empty() {
                    desired_deaths.push(i);
                }
            }
        }

        // Check all empty cells for birth
        // Only need to check cells adjacent to live cells
        let mut candidates = std::collections::HashSet::new();
        for c in &self.cells {
            let gx = c.x as usize % W;
            let gy = c.y as usize % H;
            for &(dy, dx) in &neighbour_offsets {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                if grid[ny * W + nx] == usize::MAX {
                    candidates.insert((ny, nx));
                }
            }
        }
        for (gy, gx) in candidates {
            let nbrs = live_neighbours(gy, gx);
            if nbrs.len() == 3 {
                desired_births.push((gy, gx, nbrs));
            }
        }

        // Apply population bounds by randomly trimming births/deaths
        let current = n;
        let after = current + desired_births.len() - desired_deaths.len();

        if after > pop_max {
            // Too many births — randomly trim births
            let excess = after - pop_max;
            shuffle_vec(&mut desired_births, &mut self.rng);
            desired_births.truncate(desired_births.len().saturating_sub(excess));
        } else if after < pop_min {
            // Too many deaths — randomly trim deaths
            let excess = pop_min - after;
            shuffle_vec(&mut desired_deaths, &mut self.rng);
            desired_deaths.truncate(desired_deaths.len().saturating_sub(excess));
        }

        // Mark deaths (we'll process them, removing from cells)
        let mut dying: std::collections::HashSet<usize> = desired_deaths.iter().cloned().collect();

        // Apply velocity transfers for deaths: distribute velocity equally to live neighbours
        // (excluding other dying cells)
        for &di in &dying {
            let (dvx, dvy) = (self.cells[di].vx, self.cells[di].vy);
            let gx = self.cells[di].x as usize % W;
            let gy = self.cells[di].y as usize % H;
            let receivers: Vec<usize> = live_neighbours(gy, gx).into_iter()
                .filter(|&ni| !dying.contains(&ni))
                .collect();
            if !receivers.is_empty() {
                let share = 1.0 / receivers.len() as f32;
                for &ri in &receivers {
                    self.cells[ri].vx += dvx * share;
                    self.cells[ri].vy += dvy * share;
                }
            }
        }

        // Remove dying cells (in reverse index order to preserve indices)
        let mut death_indices: Vec<usize> = dying.drain().collect();
        death_indices.sort_unstable_by(|a, b| b.cmp(a));
        for i in death_indices {
            self.cells.swap_remove(i);
        }

        // Births: place new cells with velocity = equal average of their neighbours
        // Neighbours' indices may have shifted due to swap_remove — rebuild grid
        let mut grid2 = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            grid2[yi * W + xi] = i;
        }

        for (gy, gx, old_nbr_indices) in desired_births {
            // Re-check the position is still empty (could collide with another birth)
            if grid2[gy * W + gx] != usize::MAX { continue; }

            // Recompute live neighbours from updated grid (old indices may be stale)
            let live_nbrs: Vec<usize> = neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                let idx = grid2[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect();

            // Need at least the original 3 (some may have died)
            if live_nbrs.is_empty() {
                // All neighbours died — skip birth, use old indices as fallback
                // (shouldn't normally happen)
                let _ = old_nbr_indices;
                continue;
            }

            let n_nbrs = live_nbrs.len() as f32;
            let vx = live_nbrs.iter().map(|&i| self.cells[i].vx).sum::<f32>() / n_nbrs;
            let vy = live_nbrs.iter().map(|&i| self.cells[i].vy).sum::<f32>() / n_nbrs;

            let new_idx = self.cells.len();
            self.cells.push(Cell {
                x: gx as f32 + 0.5,
                y: gy as f32 + 0.5,
                vx,
                vy,
            });
            grid2[gy * W + gx] = new_idx;
        }

        // Rebuild order for gravity shuffle
        self.order = (0..self.cells.len()).collect();
    }

    // ── Gravity step ───────────────────────────────────────────────────────
    fn gravity_step(&mut self) {
        let n = self.cells.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let mut dx = self.cells[j].x - self.cells[i].x;
                let mut dy = self.cells[j].y - self.cells[i].y;
                let hw = W as f32 / 2.0;
                let hh = H as f32 / 2.0;
                if dx >  hw { dx -= W as f32; }
                if dx < -hw { dx += W as f32; }
                if dy >  hh { dy -= H as f32; }
                if dy < -hh { dy += H as f32; }

                let r2 = dx * dx + dy * dy + self.softening * self.softening;
                let r = r2.sqrt();
                let force = self.g / r2;
                let fx = force * dx / r;
                let fy = force * dy / r;

                self.cells[i].vx += fx;
                self.cells[i].vy += fy;
                self.cells[j].vx -= fx;
                self.cells[j].vy -= fy;
            }
        }

        // Speed cap
        for c in &mut self.cells {
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            if spd > self.speed_cap {
                c.vx = c.vx / spd * self.speed_cap;
                c.vy = c.vy / spd * self.speed_cap;
            }
        }

        // Movement with reservation chaining:
        // If a cell can't move because its target is occupied, it reserves that cell.
        // If the occupant later moves away in the same tick, the reservation is fulfilled
        // and may cascade further reservations.

        // Build: cell index at each grid position
        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y as usize % H * W + c.x as usize % W] = i;
        }

        // Compute desired target for each cell
        let mut target_pos: Vec<(usize, usize, f32, f32)> = self.cells.iter().map(|c| {
            let nx = (c.x + c.vx).rem_euclid(W as f32);
            let ny = (c.y + c.vy).rem_euclid(H as f32);
            (nx as usize % W, ny as usize % H, nx, ny)
        }).collect();

        // Shuffle order for fairness
        let n = self.order.len();
        for i in (1..n).rev() {
            let j = (xoru64(&mut self.rng) as usize) % (i + 1);
            self.order.swap(i, j);
        }

        // reservation: for each grid cell, which cell index wants to move there
        let mut reservation: Vec<usize> = vec![usize::MAX; W * H];

        // First pass: try to move each cell; if blocked, record reservation
        let mut moved = vec![false; n];
        for &idx in &self.order {
            let (tx, ty, nx, ny) = target_pos[idx];
            let old_x = self.cells[idx].x as usize % W;
            let old_y = self.cells[idx].y as usize % H;

            if tx == old_x && ty == old_y {
                // Sub-pixel move, same cell
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
                moved[idx] = true;
                continue;
            }

            if grid[ty * W + tx] == usize::MAX {
                // Target free — move immediately
                grid[old_y * W + old_x] = usize::MAX;
                grid[ty * W + tx] = idx;
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
                moved[idx] = true;

                // Cascade: if anyone reserved this old cell, they can now move
                let mut freed = old_y * W + old_x;
                loop {
                    let waiter = reservation[freed];
                    if waiter == usize::MAX { break; }
                    reservation[freed] = usize::MAX;
                    let (wtx, wty, wnx, wny) = target_pos[waiter];
                    let wox = self.cells[waiter].x as usize % W;
                    let woy = self.cells[waiter].y as usize % H;
                    // freed cell should now be empty
                    grid[woy * W + wox] = usize::MAX;
                    grid[wty * W + wtx] = waiter;
                    self.cells[waiter].x = wnx;
                    self.cells[waiter].y = wny;
                    moved[waiter] = true;
                    freed = woy * W + wox; // cascade: waiter's old cell is now freed
                }
            } else {
                // Blocked — record reservation (first-come wins)
                let key = ty * W + tx;
                if reservation[key] == usize::MAX {
                    reservation[key] = idx;
                }
            }
        }
    }

    fn tick(&mut self) {
        // Run Conway only when enabled and on the right tick
        if self.conway_every > 0 && self.tick_count % self.conway_every == 0 {
            self.conway_step();
        }
        self.gravity_step();
        self.tick_count += 1;
    }

    fn paint_frame(&mut self, canvas: &mut Vec<u8>) {
        // Step 1: fade canvas.
        //   - Pixels that were live last frame: snap down by 25% (→ 75%), then slow-fade
        //   - All other pixels: slow-fade only (254/256 ≈ 99.2%)
        for py in 0..H {
            for px in 0..W {
                let i = (py * W + px) * 3;
                if self.prev_live[py * W + px] {
                    // Was live last frame — snap to 75% then slow-fade
                    canvas[i]     = (canvas[i]     as u16 * 192 / 256) as u8;
                    canvas[i + 1] = (canvas[i + 1] as u16 * 192 / 256) as u8;
                    canvas[i + 2] = (canvas[i + 2] as u16 * 192 / 256) as u8;
                } else {
                    // Slow fade only
                    canvas[i]     = (canvas[i]     as u16 * 254 / 256) as u8;
                    canvas[i + 1] = (canvas[i + 1] as u16 * 254 / 256) as u8;
                    canvas[i + 2] = (canvas[i + 2] as u16 * 254 / 256) as u8;
                }
            }
        }

        // Step 2: record which pixels are live now, then paint them at full brightness
        self.prev_live.fill(false);
        for c in &self.cells {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            let (r, g, b) = velocity_color(c.vx, c.vy, self.speed_cap);
            let i = (yi * W + xi) * 3;
            canvas[i]     = r;
            canvas[i + 1] = g;
            canvas[i + 2] = b;
            self.prev_live[yi * W + xi] = true;
        }
    }

    fn save_png(canvas: &[u8], path: &str) {
        let file = fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(BufWriter::new(file), W as u32, H as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(canvas).unwrap();
    }

    fn print_ascii(&self, label: &str) {
        println!("--- {label} (cells={}) ---", self.cells.len());
        let scale = 4usize;
        let mut grid = vec![false; (W/scale) * (H/scale)];
        for c in &self.cells {
            let xi = (c.x as usize % W) / scale;
            let yi = (c.y as usize % H) / scale;
            grid[yi * (W/scale) + xi] = true;
        }
        for y in 0..H/scale {
            let row: String = (0..W/scale).map(|x| if grid[y*(W/scale)+x] { '█' } else { '·' }).collect();
            println!("  {row}");
        }
    }

    fn stats(&self) -> String {
        let n = self.cells.len() as f32;
        let avg_spd = self.cells.iter().map(|c| (c.vx*c.vx+c.vy*c.vy).sqrt()).sum::<f32>() / n;
        let max_spd = self.cells.iter().map(|c| (c.vx*c.vx+c.vy*c.vy).sqrt()).fold(0.0f32, f32::max);
        let cx = self.cells.iter().map(|c| c.x).sum::<f32>() / n;
        let cy = self.cells.iter().map(|c| c.y).sum::<f32>() / n;
        let spread = self.cells.iter().map(|c| {
            let dx = c.x - cx; let dy = c.y - cy;
            (dx*dx+dy*dy).sqrt()
        }).sum::<f32>() / n;
        format!("pop={} avg_spd={avg_spd:.3} max={max_spd:.3} spread={spread:.1}",
            self.cells.len())
    }
}

fn shuffle_vec<T>(v: &mut Vec<T>, rng: &mut u64) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (xoru64(rng) as usize) % (i + 1);
        v.swap(i, j);
    }
}

/// Map velocity to color:
/// - Hue = direction of motion (angle of vx,vy)
/// - Saturation = speed (0=grey, 1=fully saturated)
/// - Value = 1.0 always, minimum brightness 25%
fn velocity_color(vx: f32, vy: f32, speed_cap: f32) -> (u8, u8, u8) {
    let spd = (vx * vx + vy * vy).sqrt();
    let sat = (spd / speed_cap).clamp(0.0, 1.0);
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
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn xoru64(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

fn xorf32(s: &mut u64) -> f32 {
    (xoru64(s) & 0xFFFFFF) as f32 / 0xFFFFFF as f32
}

fn run(name: &str, g: f32, softening: f32, speed_cap: f32, conway_every: usize,
       clumps: &[(f32, f32, f32, f32, f32, usize)],
       ticks: usize, snap_at: &[usize]) {
    let dir = format!("frames/{name}");
    fs::create_dir_all(&dir).unwrap();

    let mut sim = Sim::new(42, g, softening, speed_cap, conway_every, clumps);
    let mut canvas = vec![0u8; W * H * 3];
    println!("\n=== {name} | g={g} soft={softening} cap={speed_cap} conway_every={conway_every} start_pop={} ===",
        sim.cells.len());

    for tick in 0..=ticks {
        sim.paint_frame(&mut canvas);
        if snap_at.contains(&tick) {
            Sim::save_png(&canvas, &format!("{dir}/t{tick:06}.png"));
            println!("  t={tick:4}  {}", sim.stats());
        }
        if tick < ticks { sim.tick(); }
    }
}

fn main() {
    fs::create_dir_all("frames").unwrap();

    // Circular orbit parameters (N-body corrected):
    //   Each blob has ~531 cells; within-blob forces dominate at short range.
    //   High softening (15px) prevents intra-blob acceleration from hitting cap.
    //   G=0.000006, softening=15, cap=0.3
    //   At sep=184px: inter-blob accel ≈ 531 * 6e-6 / 184² ≈ 9.4e-8 px/tick² per cell
    //   Within-blob at r=2px: 6e-6 / (4 + 225) ≈ 2.6e-8 — manageable with soft=15
    //   v_circ = sqrt(531 * G * 92 / 184²) = sqrt(531 * 6e-6 * 92 / 33856) ≈ 0.029 px/tick
    //   Period ≈ 2π * 92 / 0.029 ≈ 19900 ticks → run 20000 ticks
    //   Blobs at x=100 and x=284, y=128

    // The real issue: with 531 cells/blob all doing N-body gravity, intra-blob
    // forces are enormous at close range and immediately saturate the speed cap.
    // Fix: remove speed cap entirely (set very high), use tiny G, print stats
    // to see what speeds actually develop — then set v_init to match.
    let snaps: Vec<usize> = (0..=10).map(|i| i * 50).collect();

    // Back to what looked good: G=0.003, soft=2, cap=1
    // Just cut initial velocity in half so they move slower
    // 40 frames at 10fps = 4 seconds, snap every 250 ticks over 10000
    let snaps: Vec<usize> = (0..=40).map(|i| i * 250).collect();

    // Back to D_best style: blobs moving toward each other, offset by ~26px vertically
    // Original had ivx=±0.2 which was too fast. Try ±0.05 — slow enough to curve.
    // Conway every 128, soft=2 (original), cap=1
    let snaps: Vec<usize> = (0..=40).map(|i| i * 250).collect();

    run("glancing_slow", 0.001, 1.5, 0.5, 128, &[
        ( 80.0, 115.0, 13.0,  0.05,  0.0, 0),  // left blob moving right
        (304.0, 141.0, 13.0, -0.05,  0.0, 0),  // right blob moving left, offset 26px down
    ], 10000, &snaps);
}

// [recovery] edit target not found, appending:
fn main() {
    fs::create_dir_all("frames").unwrap();

    // D_best style: two blobs moving toward each other, offset vertically.
    // This is what produced the interesting collision/slingshot behavior.
    // Original had vx=±0.2 which was too fast. Trying slower speeds.
    // 40 frames at 10fps = 4s video, snap every 120 ticks over 4800 ticks.
    let snaps: Vec<usize> = (0..=40).map(|i| i * 120).collect();

    for &(vx, conway_every) in &[(0.08f32, 0usize), (0.08, 128), (0.04, 0), (0.04, 128)] {
        let name = format!("glancing_v{vx:.2}_c{conway_every}");
        run(&name, 0.001, 1.5, 0.5, conway_every, &[
            ( 80.0, 115.0, 13.0,  vx,  0.0, 0),
            (304.0, 141.0, 13.0, -vx,  0.0, 0),
        ], 4800, &snaps);
    }
}

// [recovery] edit target not found, appending:
    // Original D_best had vx=±0.2, offset=26px — looked great but moved too fast.
    // Keep vx=±0.2, raise cap to 2.0 so gravity can still steer post-collision.
    // Vary offset to find the best slingshot angle.
    // G=0.00005 was best — spread dropped 112→92, actual attraction visible
    // Now try smaller blobs (r=6) — fewer cells = less intra-blob self-gravity
    // Also keep r=13 at same G for comparison
    // Winner: r=6, G=0.00005 — gentle deflection, beautiful trails
    // 8x frames = 320 snaps, at 30fps = ~10s video
    // 4800 ticks * 8 = 38400 ticks total, snap every 120
    let snaps: Vec<usize> = (0..=320).map(|i| i * 120).collect();

    run("beauty_r6_g5e5", 0.00005, 1.5, 0.5, 0, &[
        ( 80.0, 115.0, 6.0,  0.2,  0.0, 0),
        (304.0, 141.0, 6.0, -0.2,  0.0, 0),
    ], 38400, &snaps);
