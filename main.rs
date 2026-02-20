use std::fs;
use std::io::BufWriter;

const W: usize = 128;
const H: usize = 128;
const G: f32 = 0.5;
const SPEED_CAP: f32 = 64.0;
const SOFTENING: f32 = 1.5;

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
}

impl Sim {
    fn new_two_clumps() -> Self {
        let mut rng = 12345u64;
        let mut cells = Vec::new();

        let clumps: &[(f32, f32, f32, f32, f32, usize)] = &[
            (42.0, 64.0, 12.0,  0.0,  1.5, 80),
            (86.0, 64.0, 12.0,  0.0, -1.5, 80),
        ];

        for &(cx, cy, r, ivx, ivy, count) in clumps {
            let mut placed = 0;
            while placed < count {
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
        Sim { cells, order: (0..n).collect(), rng }
    }

    fn tick(&mut self) {
        let n = self.cells.len();

        // All-pairs gravity
        for i in 0..n {
            for j in (i + 1)..n {
                let mut dx = self.cells[j].x - self.cells[i].x;
                let mut dy = self.cells[j].y - self.cells[i].y;
                if dx >  W as f32 / 2.0 { dx -= W as f32; }
                if dx < -W as f32 / 2.0 { dx += W as f32; }
                if dy >  H as f32 / 2.0 { dy -= H as f32; }
                if dy < -H as f32 / 2.0 { dy += H as f32; }

                let r2 = dx * dx + dy * dy + SOFTENING * SOFTENING;
                let r = r2.sqrt();
                let force = G / r2;
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
            if spd > SPEED_CAP {
                c.vx = c.vx / spd * SPEED_CAP;
                c.vy = c.vy / spd * SPEED_CAP;
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
            let t = (spd / SPEED_CAP).clamp(0.0, 1.0);
            // white → orange by speed
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
        println!("  saved {path}");
    }

    fn log(&self, tick: usize) {
        let speeds: Vec<f32> = self.cells.iter()
            .map(|c| (c.vx * c.vx + c.vy * c.vy).sqrt())
            .collect();
        let avg_spd = speeds.iter().sum::<f32>() / speeds.len() as f32;
        let max_spd = speeds.iter().cloned().fold(0.0f32, f32::max);
        // center of mass
        let cx = self.cells.iter().map(|c| c.x).sum::<f32>() / self.cells.len() as f32;
        let cy = self.cells.iter().map(|c| c.y).sum::<f32>() / self.cells.len() as f32;
        println!("tick {tick:4} | cells={} | avg_spd={avg_spd:.2} max_spd={max_spd:.2} | com=({cx:.1},{cy:.1})",
            self.cells.len());
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

fn main() {
    fs::create_dir_all("frames").unwrap();
    let mut sim = Sim::new_two_clumps();

    let total_ticks = 300;
    let screenshot_ticks = [0, 10, 25, 50, 100, 150, 200, 300];

    println!("gravity sim: {}×{} grid, {} cells, G={G}, softening={SOFTENING}",
        W, H, sim.cells.len());

    for tick in 0..=total_ticks {
        if tick % 10 == 0 {
            sim.log(tick);
        }
        if screenshot_ticks.contains(&tick) {
            sim.save_png(&format!("frames/frame_{tick:04}.png"));
        }
        if tick < total_ticks {
            sim.tick();
        }
    }

    println!("done.");
}
