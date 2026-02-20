use minifb::{Key, Window, WindowOptions};

const W: usize = 128;
const H: usize = 128;
const SCALE: usize = 2; // 128×128 cells → 256×256 window
const G: f32 = 0.5;
const SPEED_CAP: f32 = 64.0;
const SOFTENING: f32 = 1.5; // avoid singularity at r≈0

struct Cell {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

struct Sim {
    cells: Vec<Cell>,
    // scratch buffer for random order iteration
    order: Vec<usize>,
    // simple xorshift rng state
    rng: u64,
}

impl Sim {
    fn new_two_clumps() -> Self {
        let mut cells = Vec::new();
        let mut rng = 12345u64;

        // Two circular clumps on opposite sides, given tangential velocities
        // so they orbit each other rather than fall straight in.
        let clumps = [
            // (center_x, center_y, radius, vx, vy, count)
            (42.0f32, 64.0, 12.0,  0.0,  1.5, 80usize),
            (86.0f32, 64.0, 12.0,  0.0, -1.5, 80usize),
        ];

        for (cx, cy, r, ivx, ivy, count) in clumps {
            let mut placed = 0;
            let mut attempts = 0;
            while placed < count && attempts < 10000 {
                attempts += 1;
                let (rx, ry) = (xorf32(&mut rng) * 2.0 - 1.0, xorf32(&mut rng) * 2.0 - 1.0);
                let (px, py) = (rx * r, ry * r);
                if px * px + py * py > r * r { continue; }
                let x = (cx + px).rem_euclid(W as f32);
                let y = (cy + py).rem_euclid(H as f32);
                // check not already occupied (grid cell)
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

        // 1. Update velocities: all-pairs gravity
        for i in 0..n {
            for j in (i + 1)..n {
                // Toroidal shortest-path delta
                let mut dx = self.cells[j].x - self.cells[i].x;
                let mut dy = self.cells[j].y - self.cells[i].y;
                if dx > W as f32 / 2.0 { dx -= W as f32; }
                if dx < -(W as f32 / 2.0) { dx += W as f32; }
                if dy > H as f32 / 2.0 { dy -= H as f32; }
                if dy < -(H as f32 / 2.0) { dy += H as f32; }

                let r2 = dx * dx + dy * dy + SOFTENING * SOFTENING;
                let r = r2.sqrt();
                let force = G / r2; // 1/r² magnitude
                let fx = force * dx / r;
                let fy = force * dy / r;

                self.cells[i].vx += fx;
                self.cells[i].vy += fy;
                self.cells[j].vx -= fx;
                self.cells[j].vy -= fy;
            }
        }

        // Clamp speeds
        for c in &mut self.cells {
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            if spd > SPEED_CAP {
                c.vx = c.vx / spd * SPEED_CAP;
                c.vy = c.vy / spd * SPEED_CAP;
            }
        }

        // 2. Move cells in random order; skip if target occupied
        // Build occupancy grid
        let mut occupied = vec![false; W * H];
        for c in &self.cells {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            occupied[yi * W + xi] = true;
        }

        // Shuffle order
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
                // Same cell, just update sub-pixel position
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
            } else if !occupied[new_yi * W + new_xi] {
                occupied[old_yi * W + old_xi] = false;
                occupied[new_yi * W + new_xi] = true;
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
            }
            // else: target occupied, don't move
        }
    }

    fn render(&self, buf: &mut Vec<u32>) {
        // Clear to black
        buf.fill(0x00000000);

        for c in &self.cells {
            let xi = c.x as usize % W;
            let yi = c.y as usize % H;
            // Color by speed
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            let t = (spd / SPEED_CAP).clamp(0.0, 1.0);
            let color = lerp_color(0x00FFFFFF, 0x00FF4400, t);

            // Each cell = SCALE×SCALE pixels in window
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let px = xi * SCALE + dx;
                    let py = yi * SCALE + dy;
                    if px < W * SCALE && py < H * SCALE {
                        buf[py * W * SCALE + px] = color;
                    }
                }
            }
        }
    }
}

fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let ar = ((a >> 16) & 0xFF) as f32;
    let ag = ((a >> 8) & 0xFF) as f32;
    let ab = (a & 0xFF) as f32;
    let br = ((b >> 16) & 0xFF) as f32;
    let bg = ((b >> 8) & 0xFF) as f32;
    let bb = (b & 0xFF) as f32;
    let r = (ar + (br - ar) * t) as u32;
    let g = (ag + (bg - ag) * t) as u32;
    let b_ = (ab + (bb - ab) * t) as u32;
    (r << 16) | (g << 8) | b_
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
    let mut sim = Sim::new_two_clumps();
    let win_w = W * SCALE;
    let win_h = H * SCALE;
    let mut buf = vec![0u32; win_w * win_h];

    let mut window = Window::new(
        "Gravity",
        win_w,
        win_h,
        WindowOptions::default(),
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut tick = 0u64;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        sim.tick();
        sim.render(&mut buf);
        window.update_with_buffer(&buf, win_w, win_h).unwrap();
        tick += 1;
        if tick % 60 == 0 {
            println!("tick {} | {} cells", tick, sim.cells.len());
        }
    }
}
