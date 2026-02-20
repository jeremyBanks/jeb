use std::fs;
use std::io::BufWriter;

const W: usize = 128;
const H: usize = 128;

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
}

impl Sim {
    fn new(rng_seed: u64, g: f32, softening: f32, speed_cap: f32,
           clumps: &[(f32, f32, f32, f32, f32, usize)]) -> Self {
        let mut rng = rng_seed;
        let mut cells = Vec::new();

        for &(cx, cy, r, ivx, ivy, count) in clumps {
            let mut placed = 0;
            let mut attempts = 0;
            while placed < count && attempts < 100_000 {
                attempts += 1;
                let rx = xorf32(&mut rng) * 2.0 - 1.0;
                let ry = xorf32(&mut rng) * 2.0 - 1.0;
                if rx * rx + ry * ry > 1.0 { continue; }
                let x = (cx + rx * r).rem_euclid(W as f32);
                let y = (cy + ry * r).rem_euclid(H as f32);
                let xi = x as usize;
                let yi = y as usize;
                if cells.iter().any(|c: &Cell| c.x as usize == xi && c.y as usize == yi) {
                    continue;
                }
                cells.push(Cell { x, y, vx: ivx, vy: ivy });
                placed += 1;
            }
        }

        let n = cells.len();
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap }
    }

    fn tick(&mut self) {
        let n = self.cells.len();

        // All-pairs 1/r² gravity
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

        // Move in random order, skip if target cell occupied
        let mut occupied = vec![false; W * H];
        for c in &self.cells {
            occupied[c.y as usize % H * W + c.x as usize % W] = true;
        }

        let n = self.order.len();
        for i in (1..n).rev() {
            let j = (xoru64(&mut self.rng) as usize) % (i + 1);
            self.order.swap(i, j);
        }

        for &idx in &self.order {
            let c = &self.cells[idx];
            let old_xi = c.x as usize % W;
            let old_yi = c.y as usize % H;
            let nx = (c.x + c.vx).rem_euclid(W as f32);
            let ny = (c.y + c.vy).rem_euclid(H as f32);
            let new_xi = nx as usize % W;
            let new_yi = ny as usize % H;

            if new_xi == old_xi && new_yi == old_yi {
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
            } else if !occupied[new_yi * W + new_xi] {
                occupied[old_yi * W + old_xi] = false;
                occupied[new_yi * W + new_xi] = true;
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
            }
        }
    }

    fn paint_frame(&self, canvas: &mut Vec<u8>) {
        // Fade existing canvas by 12.5% (multiply by 0.875 = 7/8)
        for v in canvas.iter_mut() {
            *v = (*v as u16 * 7 / 8) as u8;
        }
        // Paint live cells on top
        for c in &self.cells {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            let t = (spd / self.speed_cap).clamp(0.0, 1.0);
            // slow = blue-white, fast = orange
            let r = (255.0 * (0.5 + 0.5 * t)) as u8;
            let g = (255.0 * (0.8 - 0.5 * t)) as u8;
            let b = (255.0 * (1.0 - t)) as u8;
            let i = (yi * W + xi) * 3;
            canvas[i]     = r;
            canvas[i + 1] = g;
            canvas[i + 2] = b;
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
        // spread = mean distance from CoM
        let spread = self.cells.iter().map(|c| {
            let dx = c.x - cx; let dy = c.y - cy;
            (dx*dx+dy*dy).sqrt()
        }).sum::<f32>() / n;
        format!("avg_spd={avg_spd:.3} max={max_spd:.3} spread={spread:.1} com=({cx:.0},{cy:.0})")
    }
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

fn run(name: &str, g: f32, softening: f32, speed_cap: f32,
       clumps: &[(f32, f32, f32, f32, f32, usize)],
       ticks: usize, snap_at: &[usize]) {
    let dir = format!("frames/{name}");
    fs::create_dir_all(&dir).unwrap();

    let mut sim = Sim::new(42, g, softening, speed_cap, clumps);
    let mut canvas = vec![0u8; W * H * 3];
    println!("\n=== {name} | g={g} soft={softening} cap={speed_cap} cells={} ===",
        sim.cells.len());

    for tick in 0..=ticks {
        sim.paint_frame(&mut canvas);
        if snap_at.contains(&tick) {
            Sim::save_png(&canvas, &format!("{dir}/t{tick:04}.png"));
            sim.print_ascii(&format!("t={tick}  {}", sim.stats()));
        }
        if tick < ticks { sim.tick(); }
    }
}

fn main() {
    fs::create_dir_all("frames").unwrap();

    // Key insight: speed_cap must be ~0.5-2.0 (sub-pixel to ~2px/tick)
    // so cells actually collide and stay clumped. G must be tiny.
    // At sub-pixel speeds, 100s of ticks needed to see meaningful motion.

    let snap = &[0, 50, 100, 200, 400];

    // Two clumps orbiting — tangential velocity chosen for rough circular orbit:
    // For two equal masses separated by d=22, v_orbit ≈ sqrt(G*M/(2d))
    // With M=60 particles each, G=0.001, d=22: v ≈ sqrt(0.001*60/44) ≈ 0.037
    // Start with a few values around that
    run("orbit_gentle", 0.001, 1.5, 1.0, &[
        (42.0, 64.0, 8.0,  0.0,  0.05, 50),
        (86.0, 64.0, 8.0,  0.0, -0.05, 50),
    ], 400, snap);

    run("orbit_fast", 0.001, 1.5, 1.0, &[
        (42.0, 64.0, 8.0,  0.0,  0.15, 50),
        (86.0, 64.0, 8.0,  0.0, -0.15, 50),
    ], 400, snap);

    run("three_triangle", 0.001, 1.5, 1.0, &[
        (64.0, 30.0, 7.0,  0.12,  0.0,  35),
        (30.0, 98.0, 7.0, -0.06, -0.10, 35),
        (98.0, 98.0, 7.0, -0.06,  0.10, 35),
    ], 400, snap);

    // Dense small clumps — more particles per area, stronger local gravity
    run("dense_orbit", 0.002, 1.0, 1.0, &[
        (44.0, 64.0, 5.0,  0.0,  0.1, 20),
        (84.0, 64.0, 5.0,  0.0, -0.1, 20),
    ], 400, snap);

    // Head-on collision (no tangential velocity)
    run("collision", 0.001, 1.5, 1.0, &[
        (35.0, 64.0, 8.0,  0.08,  0.0, 50),
        (93.0, 64.0, 8.0, -0.08,  0.0, 50),
    ], 400, snap);

    // Off-center collision — glancing blow
    run("glancing", 0.001, 1.5, 1.0, &[
        (35.0, 56.0, 8.0,  0.08,  0.0, 50),
        (93.0, 72.0, 8.0, -0.08,  0.0, 50),
    ], 400, snap);
}

// [recovery] edit target not found, appending:
fn run(name: &str, g: f32, softening: f32, speed_cap: f32,
       clumps: &[(f32, f32, f32, f32, f32, usize)],
       ticks: usize, snap_at: &[usize]) {
    let dir = format!("frames/{name}");
    fs::create_dir_all(&dir).unwrap();

    let mut sim = Sim::new(42, g, softening, speed_cap, clumps);
    let mut canvas = vec![0u8; W * H * 3];

    println!("\n=== {name} | g={g} soft={softening} cap={speed_cap} cells={} ===",
        sim.cells.len());

    for tick in 0..=ticks {
        sim.paint_frame(&mut canvas);
        if snap_at.contains(&tick) {
            Sim::save_png(&canvas, &format!("{dir}/t{tick:04}.png"));
            sim.print_ascii(&format!("t={tick}  {}", sim.stats()));
        }
        if tick < ticks { sim.tick(); }
    }
}
