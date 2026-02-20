use std::fs;
use std::io::{BufWriter, Write, Read};
use std::process::Command;

const W: usize = 192;
const H: usize = 128;

// Output video settings
const OUT_W: u32 = 3840;
const OUT_H: u32 = 2160;
const FPS: u32 = 60;
const CRF: u32 = 12;
const CHUNK_FRAMES: usize = 3840; // 64s at 60fps

struct Cell {
    x: f32,
    y: f32,
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
}

impl Sim {
    fn new(rng_seed: u64, g: f32, softening: f32, speed_cap: f32, conway_every: usize, pop_band: f32,
           clumps: &[(f32, f32, f32, f32, f32, usize)]) -> Self {
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
                    // Checkerboard 50%, then randomly discard half → ~25% density
                    if xi % 2 != 0 || yi % 2 != 0 { continue; }
                    if xoru64(&mut rng) % 2 != 0 { continue; }
                    if cells.iter().any(|c: &Cell| c.x as usize == xi && c.y as usize == yi) {
                        continue;
                    }
                    cells.push(Cell { x, y, vx: ivx, vy: ivy, prev_speed: 0.0 });
                }
            }
        }
        shuffle_vec(&mut cells, &mut rng);

        // Seed 1/64th of empty cells as zero-momentum live cells
        let mut occupied = vec![false; W * H];
        for c in &cells {
            occupied[c.y as usize % H * W + c.x as usize % W] = true;
        }
        let seed_count = occupied.iter().filter(|&&v| !v).count() / 64;
        let mut seeded = 0;
        for _ in 0..W * H * 4 {
            if seeded >= seed_count { break; }
            let xi = (xoru64(&mut rng) as usize) % W;
            let yi = (xoru64(&mut rng) as usize) % H;
            let idx = yi * W + xi;
            if !occupied[idx] {
                cells.push(Cell { x: xi as f32 + 0.5, y: yi as f32 + 0.5, vx: 0.0, vy: 0.0, prev_speed: 0.0 });
                occupied[idx] = true;
                seeded += 1;
            }
        }
        shuffle_vec(&mut cells, &mut rng);

        let n = cells.len();
        let target_pop = W * H / 32;
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap, start_pop: target_pop,
              conway_every, pop_band, tick_count: 0, prev_live: vec![false; W * H] }
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
            buf.extend_from_slice(&c.x.to_le_bytes());
            buf.extend_from_slice(&c.y.to_le_bytes());
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
                       conway_every: usize, pop_band: f32)
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

        let mut cells = Vec::with_capacity(n_cells);
        for _ in 0..n_cells {
            let x  = read_f32!();
            let y  = read_f32!();
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

        let order = (0..cells.len()).collect();
        let target_pop = W * H / 32;
        let sim = Sim { cells, order, rng, g, softening, speed_cap,
                        start_pop: target_pop, conway_every, pop_band,
                        tick_count, prev_live };
        Some((sim, canvas, chunk_index))
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
        let live_neighbours = |gy: usize, gx: usize| -> Vec<usize> {
            neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
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
        shuffle_vec(&mut desired_births, &mut self.rng);

        let max_births = pop_max.saturating_sub(n);
        let max_deaths = n.saturating_sub(pop_min);
        desired_births.truncate(max_births);
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

        for (gy, gx, old_nbr_indices) in desired_births {
            if grid2[gy * W + gx] != usize::MAX { continue; }
            let live_nbrs: Vec<usize> = neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
                let idx = grid2[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect();
            if live_nbrs.is_empty() { let _ = old_nbr_indices; continue; }
            let n_nbrs = live_nbrs.len() as f32;
            let vx = live_nbrs.iter().map(|&i| self.cells[i].vx).sum::<f32>() / n_nbrs;
            let vy = live_nbrs.iter().map(|&i| self.cells[i].vy).sum::<f32>() / n_nbrs;
            let new_idx = self.cells.len();
            let birth_spd = (vx * vx + vy * vy).sqrt();
            self.cells.push(Cell { x: gx as f32 + 0.5, y: gy as f32 + 0.5, vx, vy, prev_speed: birth_spd });
            grid2[gy * W + gx] = new_idx;
        }

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

        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.y as usize % H * W + c.x as usize % W] = i;
        }

        let target_pos: Vec<(usize, usize, f32, f32)> = self.cells.iter().map(|c| {
            let nx = (c.x + c.vx).rem_euclid(W as f32);
            let ny = (c.y + c.vy).rem_euclid(H as f32);
            (nx as usize % W, ny as usize % H, nx, ny)
        }).collect();

        let n = self.order.len();
        for i in (1..n).rev() {
            let j = (xoru64(&mut self.rng) as usize) % (i + 1);
            self.order.swap(i, j);
        }

        let mut reservation: Vec<usize> = vec![usize::MAX; W * H];
        let mut moved = vec![false; n];

        for &idx in &self.order {
            let (tx, ty, nx, ny) = target_pos[idx];
            let old_x = self.cells[idx].x as usize % W;
            let old_y = self.cells[idx].y as usize % H;

            if tx == old_x && ty == old_y {
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
                moved[idx] = true;
                continue;
            }

            if grid[ty * W + tx] == usize::MAX {
                grid[old_y * W + old_x] = usize::MAX;
                grid[ty * W + tx] = idx;
                self.cells[idx].x = nx;
                self.cells[idx].y = ny;
                moved[idx] = true;

                let mut freed = old_y * W + old_x;
                loop {
                    let waiter = reservation[freed];
                    if waiter == usize::MAX { break; }
                    reservation[freed] = usize::MAX;
                    let (wtx, wty, wnx, wny) = target_pos[waiter];
                    let wox = self.cells[waiter].x as usize % W;
                    let woy = self.cells[waiter].y as usize % H;
                    grid[woy * W + wox] = usize::MAX;
                    grid[wty * W + wtx] = waiter;
                    self.cells[waiter].x = wnx;
                    self.cells[waiter].y = wny;
                    moved[waiter] = true;
                    freed = woy * W + wox;
                }
            } else {
                let key = ty * W + tx;
                if reservation[key] == usize::MAX { reservation[key] = idx; }
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

    fn paint_frame(&mut self, canvas: &mut Vec<f32>) {
        for py in 0..H {
            for px in 0..W {
                let i = (py * W + px) * 3;
                if self.prev_live[py * W + px] {
                    canvas[i]     *= 0.5;
                    canvas[i + 1] *= 0.5;
                    canvas[i + 2] *= 0.5;
                } else {
                    canvas[i]     *= 0.99875;
                    canvas[i + 1] *= 0.99875;
                    canvas[i + 2] *= 0.99875;
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
        let cx = self.cells.iter().map(|c| c.x).sum::<f32>() / n;
        let cy = self.cells.iter().map(|c| c.y).sum::<f32>() / n;
        let hw = W as f32 / 2.0; let hh = H as f32 / 2.0;
        let spread = self.cells.iter().map(|c| {
            let mut dx = c.x - cx; let mut dy = c.y - cy;
            if dx > hw { dx -= W as f32; } if dx < -hw { dx += W as f32; }
            if dy > hh { dy -= H as f32; } if dy < -hh { dy += H as f32; }
            (dx*dx+dy*dy).sqrt()
        }).sum::<f32>() / n;
        format!("pop={} avg_spd={avg_spd:.3} max={max_spd:.3} spread={spread:.1}", self.cells.len())
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
        0 => (v, t, p), 1 => (q, v, p), 2 => (p, v, t),
        3 => (p, q, v), 4 => (t, p, v), _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn xoru64(s: &mut u64) -> u64 {
    *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s
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
    // Parse --seconds N
    let args: Vec<String> = std::env::args().collect();
    let seconds: usize = {
        let pos = args.iter().position(|a| a == "--seconds")
            .expect("Usage: gravity --seconds <N>");
        args.get(pos + 1).and_then(|s| s.parse().ok())
            .expect("--seconds requires a positive integer")
    };

    let total_frames = seconds * FPS as usize;
    let n_chunks = (total_frames + CHUNK_FRAMES - 1) / CHUNK_FRAMES;

    println!("gravity: {}s × {}fps = {} frames, {} chunks of {} frames",
        seconds, FPS, total_frames, n_chunks, CHUNK_FRAMES);

    // Sim parameters
    let g           = 0.00005_f32;
    let softening   = 1.5_f32;
    let speed_cap   = 0.03125_f32;
    let conway_every = 1_usize;
    let pop_band    = 16.0_f32;

    // Four clockwise blobs, r=6 (halved again for much smaller initial pop)
    let clumps: &[(f32, f32, f32, f32, f32, usize)] = &[
        ( 48.0,  32.0, 6.0,  0.010,  0.000, 0),
        (144.0,  32.0, 6.0,  0.000,  0.010, 0),
        (144.0,  96.0, 6.0, -0.010,  0.000, 0),
        ( 48.0,  96.0, 6.0,  0.000, -0.010, 0),
    ];

    let checkpoint_path = "state/checkpoint.bin";
    let segments_dir    = "segments";
    let frames_dir      = "frames/chunk";
    let segments_file   = "segments.txt";
    let output_file     = format!("gravity_{}s.mp4", seconds);

    fs::create_dir_all(segments_dir).unwrap();
    fs::create_dir_all(frames_dir).unwrap();
    fs::create_dir_all("state").unwrap();

    // Load checkpoint or init fresh
    let (mut sim, mut canvas, start_chunk) =
        Sim::load_checkpoint(checkpoint_path, g, softening, speed_cap, conway_every, pop_band)
        .map(|(s, c, ci)| {
            println!("Resuming from checkpoint: chunk {}/{}", ci, n_chunks);
            (s, c, ci)
        })
        .unwrap_or_else(|| {
            println!("Fresh start");
            let s = Sim::new(42, g, softening, speed_cap, conway_every, pop_band, clumps);
            let c = vec![0.0f32; W * H * 3];
            (s, c, 0)
        });

    // Open/append segments list
    let mut seg_list = fs::OpenOptions::new()
        .create(true).append(true)
        .open(segments_file).unwrap();

    for chunk in start_chunk..n_chunks {
        let chunk_start_frame = chunk * CHUNK_FRAMES;
        let chunk_end_frame = ((chunk + 1) * CHUNK_FRAMES).min(total_frames);
        let this_chunk_frames = chunk_end_frame - chunk_start_frame;

        println!("\n[chunk {}/{n_chunks}] frames {}..{}", chunk+1, chunk_start_frame, chunk_end_frame);

        // Render frames for this chunk
        for local_frame in 0..this_chunk_frames {
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

    // Final concat
    println!("\nConcatenating {} segments → {output_file}", n_chunks);
    concat_segments(segments_file, &output_file);

    let size = fs::metadata(&output_file).map(|m| m.len()).unwrap_or(0);
    println!("Done! {output_file} ({:.1} MB)", size as f64 / 1_048_576.0);

    // Clean up checkpoint on successful completion
    let _ = fs::remove_file(checkpoint_path);
    println!("Checkpoint removed.");
}

// [recovery] edit target not found, appending:
use std::io::{BufWriter, Write};
