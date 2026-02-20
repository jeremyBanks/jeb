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

        // All-pairs gravity
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

        // Move in random order, skip if target occupied
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

    fn save_png(&self, path: &str) {
        let mut pixels = vec![0u8; W * H * 3];
        for c in &self.cells {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            let t = (spd / self.speed_cap).clamp(0.0, 1.0);
            let r = 255;
            let g = (255.0 * (1.0 - t * 0.73)) as u8;
            let b = (255.0 * (1.0 - t)) as u8;
            let i = (yi * W + xi) * 3;
            pixels[i]     = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
        }
        let file = fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(BufWriter::new(file), W as u32, H as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }

    fn stats(&self) -> (f32, f32, f32, f32) {
        let speeds: Vec<f32> = self.cells.iter()
            .map(|c| (c.vx * c.vx + c.vy * c.vy).sqrt())
            .collect();
        let avg_spd = speeds.iter().sum::<f32>() / speeds.len() as f32;
        let max_spd = speeds.iter().cloned().fold(0.0f32, f32::max);
        let cx = self.cells.iter().map(|c| c.x).sum::<f32>() / self.cells.len() as f32;
        let cy = self.cells.iter().map(|c| c.y).sum::<f32>() / self.cells.len() as f32;
        (avg_spd, max_spd, cx, cy)
    }

    // Measure how "interesting" the sim is:
    // reward: cells staying clumped (low spread), varied speeds, center of mass moving
    fn score_snapshot(&self) -> f32 {
        // Spread: mean distance from center of mass
        let cx = self.cells.iter().map(|c| c.x).sum::<f32>() / self.cells.len() as f32;
        let cy = self.cells.iter().map(|c| c.y).sum::<f32>() / self.cells.len() as f32;
        let spread = self.cells.iter().map(|c| {
            let dx = c.x - cx;
            let dy = c.y - cy;
            (dx*dx + dy*dy).sqrt()
        }).sum::<f32>() / self.cells.len() as f32;
        // Speed variance
        let avg_spd = self.cells.iter().map(|c| (c.vx*c.vx+c.vy*c.vy).sqrt()).sum::<f32>()
            / self.cells.len() as f32;
        let spd_var = self.cells.iter().map(|c| {
            let s = (c.vx*c.vx+c.vy*c.vy).sqrt();
            (s - avg_spd).powi(2)
        }).sum::<f32>() / self.cells.len() as f32;
        // Score: reward tight spread + speed variation
        spd_var.sqrt() - spread * 0.1
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

// Run a config for N ticks, return (score_at_end, final_sim)
fn run_config(name: &str, g: f32, softening: f32, speed_cap: f32,
              clumps: &[(f32, f32, f32, f32, f32, usize)],
              ticks: usize, snapshots: &[usize]) -> f32 {
    let dir = format!("frames/{name}");
    fs::create_dir_all(&dir).unwrap();

    let mut sim = Sim::new(12345, g, softening, speed_cap, clumps);
    let mut score_sum = 0.0f32;

    for tick in 0..=ticks {
        if snapshots.contains(&tick) {
            sim.save_png(&format!("{dir}/t{tick:04}.png"));
        }
        let s = sim.score_snapshot();
        score_sum += s;
        if tick < ticks { sim.tick(); }
    }

    let (avg_spd, max_spd, cx, cy) = sim.stats();
    let score = score_sum / ticks as f32;
    println!("  {name:<20} g={g:.3} soft={softening:.1} cells={} | avg_spd={avg_spd:.2} max={max_spd:.2} com=({cx:.0},{cy:.0}) | score={score:.3}",
        sim.cells.len());
    score
}

fn main() {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| "frames".to_string());
    fs::create_dir_all(&out_dir).unwrap();

    let snap = &[0, 20, 50, 100, 200];

    println!("=== iterating configs ===\n");

    let configs: Vec<(&str, f32, f32, f32, Vec<(f32,f32,f32,f32,f32,usize)>)> = vec![
        // name, G, softening, speed_cap, clumps: (cx, cy, radius, vx, vy, count)

        // Two clumps, gentle G, orbital tangential velocity
        ("two_clumps_slow",   0.05, 2.0, 8.0, vec![
            (42.0, 64.0, 10.0,  0.0,  0.4, 60),
            (86.0, 64.0, 10.0,  0.0, -0.4, 60),
        ]),
        ("two_clumps_med",    0.1, 2.0, 8.0, vec![
            (42.0, 64.0, 10.0,  0.0,  0.6, 60),
            (86.0, 64.0, 10.0,  0.0, -0.6, 60),
        ]),
        ("two_clumps_fast",   0.2, 2.0, 16.0, vec![
            (42.0, 64.0, 10.0,  0.0,  1.0, 60),
            (86.0, 64.0, 10.0,  0.0, -1.0, 60),
        ]),

        // Three clumps in triangle
        ("three_clumps",      0.1, 2.0, 8.0, vec![
            (64.0, 32.0, 8.0,  0.5,  0.0, 40),
            (32.0, 96.0, 8.0, -0.25, -0.4, 40),
            (96.0, 96.0, 8.0, -0.25,  0.4, 40),
        ]),

        // Four corner clumps falling inward
        ("four_corners",      0.08, 2.0, 8.0, vec![
            (20.0, 20.0, 8.0,  0.3,  0.3, 30),
            (108.0,20.0, 8.0, -0.3,  0.3, 30),
            (20.0,108.0, 8.0,  0.3, -0.3, 30),
            (108.0,108.0,8.0, -0.3, -0.3, 30),
        ]),

        // Two dense small clumps, strong G — collapse and orbit
        ("dense_pair",        0.3, 1.0, 16.0, vec![
            (44.0, 64.0, 6.0,  0.0,  1.2, 30),
            (84.0, 64.0, 6.0,  0.0, -1.2, 30),
        ]),

        // Disk collision — two clumps headed toward each other
        ("head_on",           0.1, 2.0, 8.0, vec![
            (32.0, 64.0, 10.0,  0.5,  0.0, 50),
            (96.0, 64.0, 10.0, -0.5,  0.0, 50),
        ]),

        // Ring of clumps
        ("ring_6",            0.08, 2.0, 8.0, vec![
            (64.0+40.0, 64.0,       5.0,  0.0,  0.6, 20),
            (64.0+20.0, 64.0+35.0, 5.0, -0.5,  0.3, 20),
            (64.0-20.0, 64.0+35.0, 5.0, -0.5, -0.3, 20),
            (64.0-40.0, 64.0,       5.0,  0.0, -0.6, 20),
            (64.0-20.0, 64.0-35.0, 5.0,  0.5, -0.3, 20),
            (64.0+20.0, 64.0-35.0, 5.0,  0.5,  0.3, 20),
        ]),
    ];

    let mut results: Vec<(f32, &str)> = vec![];
    for (name, g, soft, cap, clumps) in &configs {
        let score = run_config(name, *g, *soft, *cap, clumps, 200, snap);
        results.push((score, name));
    }

    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("\n=== rankings ===");
    for (score, name) in &results {
        println!("  {score:6.3}  {name}");
    }
}
