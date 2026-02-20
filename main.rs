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
        // Fade existing canvas by ~2% per tick (multiply by 0.98)
        // At 0.5px/tick avg speed, cells travel ~50px before fully faded = nice long trails
        for v in canvas.iter_mut() {
            *v = (*v as u16 * 250 / 256) as u8;
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

    // Two circles of cells, in opposite diagonal quadrants, moving on parallel paths.
    // Circle A: top-left quadrant, moving right (+x)
    // Circle B: bottom-right quadrant, moving left (-x)
    // Both paths are horizontal at y=42 and y=86 respectively — parallel, offset.
    // They'll pass each other, gravity will curve them, trails will show the arc.
    //
    // Speed: 0.5px/tick → 256 ticks to cross the grid
    // Fade: 2%/tick → ~110 ticks to fully fade → trails span ~55px = nearly half the grid
    run("parallel", 0.0008, 1.5, 2.0, &[
        (32.0, 42.0, 10.0,  0.5,  0.0, 60),  // circle A: top-left, moving right
        (96.0, 86.0, 10.0, -0.5,  0.0, 60),  // circle B: bottom-right, moving left
    ], 600, &[0, 50, 100, 150, 200, 300, 400, 600]);
}
