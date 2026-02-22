use std::fs;
use std::io::{BufWriter, Write};
use std::process::Command;
use std::sync::{Arc, OnceLock, atomic::{AtomicBool, Ordering}};

// Runtime-configurable grid resolution (set once in main before any use).
static W_CELL: OnceLock<usize> = OnceLock::new();
static H_CELL: OnceLock<usize> = OnceLock::new();
#[inline] fn W() -> usize { *W_CELL.get().expect("W not initialised") }
#[inline] fn H() -> usize { *H_CELL.get().expect("H not initialised") }
#[inline] fn OUT_W() -> u32 { W() as u32 }
#[inline] fn OUT_H() -> u32 { H() as u32 }
const FPS: u32 = 60;
const CRF: u32 = 12;
const CHUNK_FRAMES: usize = 4096; // 68.3s at 60fps → 64 segments for a 64³-frame run

// ── Audio constants ────────────────────────────────────────────────────────
const SAMPLE_RATE: u32         = 44100;
const SAMPLES_PER_FRAME: usize = 735;    // 44100 / 60, truncated (acceptable drift)
const AUDIO_AMP_SCALE: f32     = 0.0008; // per-voice scale (9 voices; tanh handles headroom)

// 3×3 spatial grid — 9 voices, one per screen region.
// Pitch: C major pentatonic across 2 octaves, A3–E5, warm-bright range (~220–660 Hz).
// Layout: top row = highest pitch, bottom row = lowest. Left-to-right within row = ascending.
//   row 0 (top):    C5(523) D5(587) E5(659)  — indices 0,1,2
//   row 1 (mid):    E4(330) G4(392) A4(440)  — indices 3,4,5
//   row 2 (bot):    A3(220) C4(262) D4(294)  — indices 6,7,8
const REGION_FREQS: [f32; 9] = [
    523.25, 587.33, 659.25,  // top row
    329.63, 392.00, 440.00,  // mid row
    220.00, 261.63, 293.66,  // bot row
];
// Pan per column: mild stereo (col 0=left, col 1=center, col 2=right)
const REGION_PAN:  [f32; 3] = [-0.30, 0.0, 0.30];
// Reverb send per row: top=airy, mid=neutral, bot=dry
const REGION_REVERB: [f32; 3] = [0.67, 0.58, 0.48];

const PITCH_BEND_MAX:  f32 = 50.0;   // cents ±   (CoG-x drives ±0.5 semitone)
const FILTER_BRIGHT:   f32 = 0.248;  // one-pole LP coeff ≈ 2000 Hz (active/sparse)
const FILTER_WARM:     f32 = 0.055;  // one-pole LP coeff ≈ 400 Hz  (dense/settled)

// Slew rates per frame: val += (target - val) * α
const SLEW_AMP:    f32 = 0.20;  // amplitude — lighter than CoG so rhythm comes through
const SLEW_COG:    f32 = 0.25;  // CoG x/y  — ~4 ticks ≈ 1/15 s
const SLEW_BEND:   f32 = 0.15;  // pitch bend from CoG-x
const SLEW_FILTER: f32 = 0.20;  // filter cutoff from CoG-y

/// Per-frame raw stats accumulated for one 3×3 spatial region (cleared each frame).
#[derive(Default, Clone, Copy)]
struct RegionStats {
    speed_sum:  f32,  // Σ speed of all cells in region
    cell_count: f32,  // number of cells in region
    cog_x_sum:  f32,  // Σ px (for CoG)
    cog_y_sum:  f32,  // Σ py (for CoG)
}

/// One persistent spatial voice — always active, amplitude→0 when idle.
struct RegionVoice {
    base_freq:    f32,  // fixed pitch for this region
    pan:          f32,  // fixed pan from column (-0.3 / 0 / +0.3)
    reverb_send:  f32,  // fixed reverb from row
    phase:        f32,  // oscillator phase
    filter_state: f32,  // one-pole LP state
    // EMA-smoothed values
    amplitude:    f32,  // pop × avg_speed, normalised
    cog_x:        f32,  // smoothed CoG x within region (0..1)
    cog_y:        f32,  // smoothed CoG y within region (0..1)
    // derived, slewed
    pitch_bend:   f32,  // cents, from cog_x
    filter_coeff: f32,  // LP cutoff coeff, from cog_y
}
impl RegionVoice {
    fn new(base_freq: f32, pan: f32, reverb_send: f32) -> Self {
        RegionVoice {
            base_freq, pan, reverb_send,
            phase: 0.0, filter_state: 0.0,
            amplitude: 0.0, cog_x: 0.5, cog_y: 0.5,
            pitch_bend: 0.0, filter_coeff: FILTER_WARM,
        }
    }
}

struct Cell {
    px: f32,   // continuous world position, x ∈ [0, W)
    py: f32,   // continuous world position, y ∈ [0, H)
    vx: f32,
    vy: f32,
    prev_speed: f32,
    id: u64,      // persistent identity — travels with the cell
    moved: bool,  // true if cell changed grid square this tick
}
impl Cell {
    #[inline] fn gx(&self) -> usize { (self.px.floor() as i32).rem_euclid(W as i32) as usize }
    #[inline] fn gy(&self) -> usize { (self.py.floor() as i32).rem_euclid(H as i32) as usize }
    #[inline] fn in_bounds(&self) -> bool {
        let x = self.px.floor() as i32;
        let y = self.py.floor() as i32;
        x >= 0 && x < W as i32 && y >= 0 && y < H as i32
    }
}

// ── Reverb (Schroeder-style: 4 parallel combs + 2 serial all-passes) ──────────
// Delay times are prime multiples of SAMPLES_PER_FRAME (735 at 44100/60fps).
// This syncs reverb tails to the video frame rate and avoids inter-line aliasing.
struct CombFilter { buf: Vec<f32>, idx: usize, feedback: f32, lp: f32 }
impl CombFilter {
    fn new(n_frames: usize, feedback: f32) -> Self {
        CombFilter { buf: vec![0.0; n_frames * SAMPLES_PER_FRAME], idx: 0, feedback, lp: 0.0 }
    }
    fn process(&mut self, inp: f32) -> f32 {
        let out = self.buf[self.idx];
        self.lp = out * 0.5 + self.lp * 0.5; // gentle HF rolloff in feedback
        self.buf[self.idx] = inp + self.lp * self.feedback;
        self.idx = (self.idx + 1) % self.buf.len();
        out
    }
}

struct AllPass { buf: Vec<f32>, idx: usize, feedback: f32 }
impl AllPass {
    fn new(n_frames: usize, feedback: f32) -> Self {
        AllPass { buf: vec![0.0; n_frames * SAMPLES_PER_FRAME], idx: 0, feedback }
    }
    fn process(&mut self, inp: f32) -> f32 {
        let delayed = self.buf[self.idx];
        let out = delayed - self.feedback * inp;
        self.buf[self.idx] = inp + self.feedback * delayed;
        self.idx = (self.idx + 1) % self.buf.len();
        out
    }
}

struct Reverb {
    combs: [CombFilter; 4],   // prime-frame delays: 11, 13, 17, 19
    allpasses: [AllPass; 2],  // 5, 3 frames
    wet: f32,
    out_lp: f32,              // global warmth LP — rolls off harshness above ~2.5kHz
}
impl Reverb {
    fn new() -> Self {
        Reverb {
            combs: [
                CombFilter::new(11, 0.84),
                CombFilter::new(13, 0.84),
                CombFilter::new(17, 0.80),
                CombFilter::new(19, 0.80),
            ],
            allpasses: [
                AllPass::new(5, 0.5),
                AllPass::new(3, 0.5),
            ],
            wet: 0.40,   // was 0.28 — more space/softness
            out_lp: 0.0,
        }
    }
    fn process(&mut self, dry: f32) -> f32 {
        let comb_sum = self.combs.iter_mut().map(|c| c.process(dry)).sum::<f32>() * 0.25;
        let ap1 = self.allpasses[0].process(comb_sum);
        let ap2 = self.allpasses[1].process(ap1);
        let mixed = dry * (1.0 - self.wet) + ap2 * self.wet;
        // One-pole LP at ~2.5kHz: coeff = 1 - exp(-2π×2500/44100) ≈ 0.30
        // Rolls off harshness, makes everything warmer without killing clarity
        self.out_lp += (mixed - self.out_lp) * 0.30;
        self.out_lp
    }
}

struct Sim {
    cells: Vec<Cell>,
    order: Vec<usize>,
    rng: u64,
    g: f32,
    softening: f32,
    speed_cap: f32,
    start_pop: usize,
    pop_band: f32,
    rate_limit: usize,
    conway_every: usize, // fire Conway every N ticks (1 = every tick, 4 = every 4th tick)
    tick_count: usize,
    prev_live: Vec<bool>,
    wrap_x: bool,  // toroidal wrapping on x-axis (horizontal)
    wrap_y: bool,  // toroidal wrapping on y-axis (vertical)
    bounce_x: bool, // reflect velocity at x boundaries (gravity: no-wrap force)
    bounce_y: bool, // reflect velocity at y boundaries (gravity: no-wrap force)
    steer: bool,   // counter-rotate velocity to compensate discrete-move angular error
    dampen_x: f32, // fraction of COM horizontal velocity removed per tick (0=off, 0.125=fast)
    dampen_y: f32, // fraction of COM vertical   velocity removed per tick (0=off, 0.125=fast)
    conway_births: usize,  // cumulative Conway births
    conway_deaths: usize,  // cumulative Conway deaths
    next_id: u64,
    region_stats:  [RegionStats; 9],  // accumulated per-frame, cleared after generate_audio
    region_voices: [RegionVoice; 9], // persistent spatial voices (3×3 grid)
    reverb: Reverb,
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
            px: f32, py: f32, g: f32, softening: f32, wrap_x: bool, wrap_y: bool) -> (f32, f32) {
    let node = &nodes[node_idx];
    if node.body == -2 { return (0.0, 0.0); } // empty node
    let raw_dx = node.com_x - px;
    let raw_dy = node.com_y - py;
    let dx = if wrap_x { min_image(raw_dx, W as f32) } else { raw_dx };
    let dy = if wrap_y { min_image(raw_dy, H as f32) } else { raw_dy };
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
            let (cfx, cfy) = qt_force(nodes, ch as usize, body, px, py, g, softening, wrap_x, wrap_y);
            fx += cfx;
            fy += cfy;
        }
    }
    (fx, fy)
}
// ──────────────────────────────────────────────────────────────────────────

impl Sim {
    fn new(rng_seed: u64, g: f32, softening: f32, speed_cap: f32, pop_band: f32,
           rate_limit: usize, conway_every: usize, seed_density_inv: usize, target_pop: usize,
           wrap_x: bool, wrap_y: bool, bounce_x: bool, bounce_y: bool, steer: bool,
           dampen_x: f32, dampen_y: f32, init_vel: &str, circles: usize, vel_scale: f32) -> Self {
        use std::f32::consts::PI;
        let mut rng = rng_seed;
        let mut next_id: u64 = 1;
        let mut cells: Vec<Cell> = Vec::new();
        let mut occupied = vec![false; W * H];

        // Helper: compute velocity for a seeded cell at grid (xi, yi).
        let cx_global = W as f32 / 2.0;
        let cy_global = H as f32 / 2.0;
        let aspect = W as f32 / H as f32; // e.g. 256/160 = 1.6
        let make_vel = |xi: usize, yi: usize, rng: &mut u64| -> (f32, f32) {
            let (vx, vy) = match init_vel {
                "swirl" => {
                    if xi < W / 2 && yi < H / 2 {
                        (xorf32(rng) * 0.75 - 0.25, (xorf32(rng) - 0.5) * 0.25)
                    } else if xi >= W / 2 && yi >= H / 2 {
                        (xorf32(rng) * 0.75 - 0.5,  (xorf32(rng) - 0.5) * 0.25)
                    } else if xi >= W / 2 {
                        ((xorf32(rng) - 0.5) * 0.25, (xorf32(rng) - 0.5) * 0.25)
                    } else {
                        ((xorf32(rng) - 0.5) * 0.25, xorf32(rng) * 0.25)
                    }
                }
                "random" => ((xorf32(rng) - 0.5) * 0.5, (xorf32(rng) - 0.5) * 0.5),
                "spin" => {
                    let dx = xi as f32 + 0.5 - cx_global;
                    let dy = yi as f32 + 0.5 - cy_global;
                    let r = (dx*dx + dy*dy).sqrt().max(1.0);
                    let scale = (r / (cx_global.min(cy_global))).min(1.0) * 0.5;
                    (-dy/r * scale + (xorf32(rng)-0.5)*0.1, dx/r * scale + (xorf32(rng)-0.5)*0.1)
                }
                "spin-ccw" => {
                    let dx = xi as f32 + 0.5 - cx_global;
                    let dy = yi as f32 + 0.5 - cy_global;
                    let r = (dx*dx + dy*dy).sqrt().max(1.0);
                    let scale = (r / (cx_global.min(cy_global))).min(1.0) * 0.5;
                    (dy/r * scale + (xorf32(rng)-0.5)*0.1, -dx/r * scale + (xorf32(rng)-0.5)*0.1)
                }
                "spin-flat" => {
                    // spin + aspect-ratio vx scaling + vy scaled down (very flat — 1/8 of original 0.75)
                    // (vx aspect scaling is applied at closure return; vy *= 0.09375 here)
                    let dx = xi as f32 + 0.5 - cx_global;
                    let dy = yi as f32 + 0.5 - cy_global;
                    let r = (dx*dx + dy*dy).sqrt().max(1.0);
                    let scale = (r / (cx_global.min(cy_global))).min(1.0) * 0.5;
                    let vx = -dy/r * scale + (xorf32(rng)-0.5)*0.1;
                    let vy = (dx/r * scale + (xorf32(rng)-0.5)*0.1) * 0.09375;
                    (vx, vy)
                }
                "radial-out" => {
                    let dx = xi as f32 + 0.5 - cx_global;
                    let dy = yi as f32 + 0.5 - cy_global;
                    let r = (dx*dx + dy*dy).sqrt().max(1.0);
                    (dx/r * 0.4 + (xorf32(rng)-0.5)*0.1, dy/r * 0.4 + (xorf32(rng)-0.5)*0.1)
                }
                "zero" | _ => (0.0, 0.0),
            };
            (vx * aspect, vy)
        };

        if circles > 0 {
            // ── Circle placement ────────────────────────────────────────────
            // Cells are distributed evenly across circles (off-by-one handled):
            //   first (target_pop % circles) circles get one extra cell.
            // Each circle is filled at ~50% average density by iterating grid
            // points in order of distance from centre (closest first) and
            // flipping a 50% coin at each point until the target count is placed.
            // Radius is sized so ≈2× target cells fit inside (at 50% fill rate).
            // Circle centres are chosen greedily to maximise minimum distance
            // from canvas walls and from each other (tie-break: closer to centre).
            let base_cells  = (target_pop / circles).max(1);
            let extra_circles = target_pop % circles; // first N circles get base+1
            let radius = ((2.0 * base_cells as f32 / PI).sqrt()).max(4.0)
                          .min((W.min(H) as f32) * 0.45 / (circles as f32).sqrt());
            let margin = radius + 1.0;

            // Candidate grid: full axis range when wrapped, margin-inset when not.
            let x_min = if wrap_x { 0.0 } else { margin };
            let x_max = if wrap_x { W as f32 } else { W as f32 - margin };
            let y_min = if wrap_y { 0.0 } else { margin };
            let y_max = if wrap_y { H as f32 } else { H as f32 - margin };
            let mut candidates: Vec<(f32, f32)> = Vec::new();
            let mut cx = x_min;
            while cx <= x_max {
                let mut cy = y_min;
                while cy <= y_max {
                    candidates.push((cx, cy));
                    cy += 2.0;
                }
                cx += 2.0;
            }

            // Score: maximise min-distance to nearest other circle (wrap-aware) and walls
            // (walls only count for non-wrapped axes). Tie-break toward canvas centre.
            let score = |px: f32, py: f32, chosen: &[(f32, f32)]| -> f32 {
                // Wall clearance only applies on non-wrapped axes.
                let wall_x = if wrap_x { f32::INFINITY } else { px.min(W as f32 - px) };
                let wall_y = if wrap_y { f32::INFINITY } else { py.min(H as f32 - py) };
                let wall = wall_x.min(wall_y);
                // Wrap-aware distance to nearest chosen circle.
                let nbr = chosen.iter()
                    .map(|&(qx, qy)| {
                        let dx_r = (px - qx).abs();
                        let dy_r = (py - qy).abs();
                        let dx = if wrap_x { dx_r.min(W as f32 - dx_r) } else { dx_r };
                        let dy = if wrap_y { dy_r.min(H as f32 - dy_r) } else { dy_r };
                        (dx*dx + dy*dy).sqrt()
                    })
                    .fold(f32::INFINITY, f32::min);
                let centre_pen = if wrap_x && wrap_y { 0.0 }
                    else { ((px - cx_global).powi(2) + (py - cy_global).powi(2)).sqrt() * 0.001 };
                wall.min(nbr) - centre_pen
            };

            let mut centres: Vec<(f32, f32)> = Vec::with_capacity(circles);
            for _ in 0..circles {
                if let Some(&best) = candidates.iter()
                    .max_by(|&&a, &&b| score(a.0, a.1, &centres)
                        .partial_cmp(&score(b.0, b.1, &centres)).unwrap())
                {
                    centres.push(best);
                }
            }

            // Fill each disk using distance-sorted grid walk with 50% coin flip.
            // Points are visited closest-to-centre first; a coin flip decides
            // whether each point is populated.  We continue until target count
            // is placed.  If the inner radius is exhausted before the target is
            // reached (rare — variance of binomial), a fallback pass places all
            // remaining unoccupied points unconditionally.
            for (ci, &(disk_cx, disk_cy)) in centres.iter().enumerate() {
                let cells_this_circle = base_cells + if ci < extra_circles { 1 } else { 0 };

                // Collect integer grid points within a search radius (1.5× for buffer).
                let r_search = radius * 1.5;
                let r_sq     = r_search * r_search;
                let r_ceil   = r_search.ceil() as isize;
                let mut pts: Vec<(usize, usize, f32)> = Vec::new();
                for dy in -r_ceil..=r_ceil {
                    for dx in -r_ceil..=r_ceil {
                        let d2 = (dx as f32).powi(2) + (dy as f32).powi(2);
                        if d2 > r_sq { continue; }
                        let xi_i = disk_cx as isize + dx;
                        let yi_i = disk_cy as isize + dy;
                        if xi_i < 0 || xi_i >= W as isize { continue; }
                        if yi_i < 0 || yi_i >= H as isize { continue; }
                        let xi = xi_i as usize;
                        let yi = yi_i as usize;
                        if !occupied[yi * W + xi] {
                            pts.push((xi, yi, d2));
                        }
                    }
                }
                // Sort by distance from centre (closest first).
                pts.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

                let mut placed = 0usize;

                // Primary pass: 50% coin flip at each point in distance order.
                for &(xi, yi, _) in &pts {
                    if placed >= cells_this_circle { break; }
                    if occupied[yi * W + xi] { continue; }
                    if xoru64(&mut rng) & 1 == 0 { continue; } // 50% skip
                    let (vx, vy) = make_vel(xi, yi, &mut rng);
                    let (vx, vy) = (vx * vel_scale, vy * vel_scale);
                    cells.push(Cell { px: xi as f32 + 0.5, py: yi as f32 + 0.5, vx, vy,
                                      prev_speed: 0.0, id: next_id, moved: false });
                    next_id += 1;
                    occupied[yi * W + xi] = true;
                    placed += 1;
                }

                // Fallback pass: fill remaining slots from inner points outward.
                if placed < cells_this_circle {
                    for &(xi, yi, _) in &pts {
                        if placed >= cells_this_circle { break; }
                        if occupied[yi * W + xi] { continue; }
                        let (vx, vy) = make_vel(xi, yi, &mut rng);
                        let (vx, vy) = (vx * vel_scale, vy * vel_scale);
                        cells.push(Cell { px: xi as f32 + 0.5, py: yi as f32 + 0.5, vx, vy,
                                          prev_speed: 0.0, id: next_id, moved: false });
                        next_id += 1;
                        occupied[yi * W + xi] = true;
                        placed += 1;
                    }
                }
            }
        } else {
        // ── Default: random scatter (original behaviour) ─────────────────────
        let seed_count = if seed_density_inv > 0 {
            (W * H) / seed_density_inv
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
                cells.push(Cell { px: xi as f32 + 0.5, py: yi as f32 + 0.5, vx, vy, prev_speed: 0.0, id: next_id, moved: false });
                next_id += 1;
                occupied[idx] = true;
                seeded += 1;
            }
        }
        shuffle_vec(&mut cells, &mut rng);

        let n = cells.len();
        let target_pop = W * H / 32;
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap, start_pop: target_pop,
              pop_band, rate_limit, conway_every, tick_count: 0, prev_live: vec![false; W * H], wrap_x, wrap_y, bounce_x, bounce_y, steer, dampen_x, dampen_y,
              conway_births: 0, conway_deaths: 0, next_id, voice_pool: HashMap::new() }
    }

    // ── Checkpoint save/load ───────────────────────────────────────────────
    fn save_checkpoint(&self, canvas: &[f32], chunk_index: usize, path: &str) {
        let mut buf: Vec<u8> = Vec::new();
        // header
        buf.extend_from_slice(&(self.cells.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.rng.to_le_bytes());
        buf.extend_from_slice(&(self.tick_count as u64).to_le_bytes());
        buf.extend_from_slice(&(chunk_index as u64).to_le_bytes());
        buf.extend_from_slice(&self.next_id.to_le_bytes());
        // cells
        for c in &self.cells {
            buf.extend_from_slice(&c.px.to_le_bytes());
            buf.extend_from_slice(&c.py.to_le_bytes());
            buf.extend_from_slice(&c.vx.to_le_bytes());
            buf.extend_from_slice(&c.vy.to_le_bytes());
            buf.extend_from_slice(&c.prev_speed.to_le_bytes());
            buf.extend_from_slice(&c.id.to_le_bytes());
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
                       pop_band: f32, rate_limit: usize, conway_every: usize,
                       _seed_density_inv: usize,
                       target_pop: usize, wrap_x: bool, wrap_y: bool,
                       bounce_x: bool, bounce_y: bool, steer: bool,
                       dampen_x: f32, dampen_y: f32)
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
        let next_id = read_u64!();

        let mut cells = Vec::with_capacity(n_cells);
        for _ in 0..n_cells {
            let px = read_f32!();
            let py = read_f32!();
            let vx = read_f32!();
            let vy = read_f32!();
            let ps = read_f32!();
            let id = read_u64!();
            cells.push(Cell { px, py, vx, vy, prev_speed: ps, id, moved: false });
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
            prev_live_rebuilt[c.gy() * W + c.gx()] = true;
        }
        let order = (0..cells.len()).collect();
        let target_pop = W * H / 32;
        let sim = Sim { cells, order, rng, g, softening, speed_cap,
                        start_pop: target_pop, pop_band, rate_limit, conway_every,
                        tick_count, prev_live: prev_live_rebuilt, wrap_x, wrap_y, bounce_x, bounce_y, steer, dampen_x, dampen_y,
                        conway_births: 0, conway_deaths: 0,
                        next_id, voice_pool: HashMap::new() };
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

        // Conway runs at a fixed rate regardless of cell speed.
        // Pop-band alone throttles births/deaths (cells can only be born up to pop_max,
        // killed down to pop_min). No speed-based shutoff.
        let rate_limit = self.rate_limit;
        // Hard cutoff at band edges: inside the band Conway runs freely and population
        // floats naturally. We only block births when at pop_max, deaths when at pop_min.
        let max_births = if n >= pop_max { 0 } else { rate_limit };
        let max_deaths = if n <= pop_min { 0 } else { rate_limit };
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
        self.conway_deaths += death_indices.len();
        for i in death_indices { self.cells.swap_remove(i); }

        let mut grid2 = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid2[c.y as usize % H * W + c.x as usize % W] = i;
        }

        // Uniform birth selection: shuffle candidates, take first max_births.
        let mut birth_indices: Vec<usize> = desired_births.iter()
            .enumerate()
            .filter_map(|(i, (gy, gx, _))| {
                if grid2[gy * W + gx] != usize::MAX { return None; }
                Some(i)
            })
            .collect();
        shuffle_vec(&mut birth_indices, &mut self.rng);

        for bi in birth_indices.into_iter().take(max_births) {
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
        // Reset moved flag each tick — only set for cells that change grid square
        for c in &mut self.cells { c.moved = false; }
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



        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.gy() * W + c.gx()] = i;
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
        if self.tick_count % self.conway_every == 0 {
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
        let live_neighbours = |gy: usize, gx: usize| -> Vec<usize> {
            neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let ny = ((gy as i32 + dy).rem_euclid(H as i32)) as usize;
                let nx = ((gx as i32 + dx).rem_euclid(W as i32)) as usize;
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

    fn paint_frame(&mut self, canvas: &mut Vec<f32>, palette: &PaletteMode) {
        for py in 0..H {
            for px in 0..W {
                let i = (py * W + px) * 3;
                if self.prev_live[py * W + px] {
                    canvas[i]     *= 0.5;
                    canvas[i + 1] *= 0.5;
                    canvas[i + 2] *= 0.5;
                } else {
                    canvas[i]     *= 0.999534; // fade rate (doubled from 0.999767)
                    canvas[i + 1] *= 0.999534;
                    canvas[i + 2] *= 0.999534;
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
        format!("pop={} births={} deaths={} avg_spd={avg_spd:.3} max={max_spd:.3} spread={spread:.1} com=({cx:.1},{cy:.1})",
            self.cells.len(), self.conway_births, self.conway_deaths)
    }
}

fn shuffle_vec<T>(v: &mut Vec<T>, rng: &mut u64) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (xoru64(rng) as usize) % (i + 1);
        v.swap(i, j);
    }
}

// Returns Oklab (L, a, b) for a cell's velocity — stored directly in canvas, no RGB conversion here.
// L: 0.45 (still) → 0.75 (fast); C: 0.0 (still) → 0.20 (fast); H: velocity direction angle.
// ── Palette system ────────────────────────────────────────────────────────────
// Hot-reload: binary reads /tmp/gravity_palette at the start of each chunk.
// File contains a single palette name: "classic" | "jeremy"
// If file is absent or unrecognised, falls back to "classic".
// "classic" = original uniform hue wheel (exact same behaviour as before).
// "jeremy"  = gravity-well hue biasing toward a curated palette; same L/C ramp.

#[derive(Clone, Debug)]
enum PaletteMode {
    /// Original: speed→L/C, direction→hue uniformly.
    Classic,
    /// Gravity-well hue biasing toward Jeremy's palette anchors.
    /// pull ∈ [0,1]: 0 = classic, 1 = maximum bias.
    /// sigma_rad: angular half-width of each well in radians (~0.7 ≈ 40°).
    Jeremy { pull: f32, sigma_rad: f32 },
}

fn load_palette() -> PaletteMode {
    let raw = std::fs::read_to_string("/tmp/gravity_palette")
        .unwrap_or_default();
    let s = raw.trim().to_lowercase();
    if s.starts_with("jeremy") {
        // Optional: "jeremy pull=0.8 sigma=0.6"
        let pull = s.split("pull=").nth(1)
            .and_then(|v| v.split_whitespace().next())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.85_f32);
        let sigma = s.split("sigma=").nth(1)
            .and_then(|v| v.split_whitespace().next())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.70_f32);  // ~40°
        PaletteMode::Jeremy { pull, sigma_rad: sigma }
    } else {
        PaletteMode::Classic
    }
}

fn velocity_color_oklab(vx: f32, vy: f32, px: f32, py: f32, speed_cap: f32, palette: &PaletteMode) -> (f32, f32, f32) {
    use std::f32::consts::PI;
    let spd = (vx * vx + vy * vy).sqrt();

    match palette {
        PaletteMode::Classic => {
            // Original behaviour — unchanged.
            let t = (spd / (speed_cap * 0.5)).clamp(0.0, 1.0);
            let l = 0.45 + 0.30 * t;
            let c = 0.20 * t;
            let h = vy.atan2(vx);
            (l, c * h.cos(), c * h.sin())
        }

        PaletteMode::Jeremy { pull, sigma_rad } => {
            // Anchor hues in radians (Oklch atan2 convention, −π..π).
            // Derived from Jeremy's palette: FF6118 FFC01F 635BFF 533AFD F44BCC EA2261
            //   orange≈40°  gold≈80°  periwinkle≈274°  violet≈280°  pink≈325°  rose≈5°
            const ANCHORS: [f32; 6] = [
                 0.698,   // FF6118  orange  ~40°
                 1.396,   // FFC01F  gold    ~80°
                -1.501,   // 635BFF  periwinkle  ~274° (= −86°)
                -1.396,   // 533AFD  violet  ~280° (= −80°)
                -0.611,   // F44BCC  pink    ~325° (= −35°)
                 0.087,   // EA2261  rose    ~5°
            ];

            let h_nat = vy.atan2(vx);   // natural hue from velocity direction
            let s2    = sigma_rad * sigma_rad;

            // Each anchor exerts a pull ∝ weight × angular displacement.
            // No snapping: force is continuous and zero when sitting on an anchor.
            let force: f32 = ANCHORS.iter().map(|&h_i| {
                let mut d = h_i - h_nat;
                // Wrap angular distance to [−π, π]
                if d >  PI { d -= 2.0 * PI; }
                if d < -PI { d += 2.0 * PI; }
                let w = s2 / (d * d + s2);
                w * d
            }).sum();

            let h_biased = h_nat + pull * force;

            // Speed ramp: dark navy (061B31, L≈0.15) → full chroma at speed_cap.
            // Slightly brighter and more saturated than classic to suit the palette.
            let t = (spd / speed_cap).clamp(0.0, 1.0);
            let l = 0.15 + 0.60 * t;
            let c = 0.24 * t;
            (l, c * h_biased.cos(), c * h_biased.sin())
        }
    }
}

fn oklab_to_srgb(l: f32, a: f32, b: f32) -> (u8, u8, u8) {
    // Oklab → LMS (cube roots)
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    // LMS → linear sRGB
    let r_lin =  4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g_lin = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let b_lin = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;
    // Linear sRGB → gamma-corrected u8 (clamp handles out-of-gamut)
    let gamma = |x: f32| -> u8 {
        let x = x.clamp(0.0, 1.0);
        let g = if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        (g * 255.0).round() as u8
    };
    (gamma(r_lin), gamma(g_lin), gamma(b_lin))
}

fn xoru64(s: &mut u64) -> u64 {
    *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s
}

fn xorf32(s: &mut u64) -> f32 {
    (xoru64(s) & 0xFFFFFF) as f32 / 0xFFFFFF as f32
}

fn encode_chunk(frames_dir: &str, seg_path: &str, n_frames: usize, tile_2x2: bool) {
    // ffmpeg glob requires sorted files — they're zero-padded so glob order = numeric order
    let ow = OUT_W * 2;
    let oh = OUT_H * 2;
    let status = if tile_2x2 {
        // Tile the frame 2×2 then scale back to normal output size (each copy is half-size).
        let fc = format!(
            "[0:v]split=4[a][b][c][d];[a][b]hstack[top];[c][d]hstack[bot];[top][bot]vstack[tiled];[tiled]scale={ow}:{oh}:flags=neighbor[out]"
        );
        Command::new("ffmpeg")
            .args([
                "-y",
                "-framerate", &FPS.to_string(),
                "-pattern_type", "glob",
                "-i", &format!("{frames_dir}/*.png"),
                "-filter_complex", &fc,
                "-map", "[out]",
                "-c:v", "libx264",
                "-crf", &CRF.to_string(),
                "-pix_fmt", "yuv420p",
                seg_path,
            ])
            .status()
            .expect("ffmpeg failed")
    } else {
        let scale = format!("scale={ow}:{oh}:flags=neighbor");
        Command::new("ffmpeg")
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
            .expect("ffmpeg failed")
    };
    assert!(status.success(), "ffmpeg exited non-zero for {seg_path}");
    println!("  encoded {n_frames} frames → {seg_path}");
}

// Write raw f32le PCM, mux with video segment in-place.
fn mux_audio_into_segment(seg_path: &str, audio: &[f32]) {
    let pcm_path = format!("{seg_path}.pcm");
    // Write f32 little-endian samples
    let bytes: Vec<u8> = audio.iter().flat_map(|&s| s.to_le_bytes()).collect();
    fs::write(&pcm_path, &bytes).expect("write pcm");

    let muxed = format!("{seg_path}.muxed.mp4");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", seg_path,                        // video-only segment
            "-f", "f32le", "-ar", "44100", "-ac", "2",
            "-i", &pcm_path,                        // raw PCM audio
            "-c:v", "copy",
            "-c:a", "aac", "-b:a", "128k",
            "-shortest",
            &muxed,
        ])
        .status()
        .expect("ffmpeg mux failed");

    if status.success() {
        fs::rename(&muxed, seg_path).expect("rename muxed");
    } else {
        eprintln!("  [audio] mux failed for {seg_path}, keeping video-only");
        let _ = fs::remove_file(&muxed);
    }
    let _ = fs::remove_file(&pcm_path);
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
    // Re-encode video with NN upscale; copy audio stream from muxed segments
    let status = Command::new("ffmpeg")
        .args([
            "-y", "-f", "concat", "-safe", "0", "-i", segments_file,
            "-vf", "scale=2048:1280:flags=neighbor",
            "-c:v", "libx264", "-crf", "12", "-preset", "fast",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac", "-b:a", "128k",
            output,
        ])
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
        .expect("Usage: gravity --seconds <N> [--seed <N>] [--seed-density <1/N>] [--epilogue]");
    let do_epilogue = args.iter().any(|a| a == "--epilogue");
    let headless    = args.iter().any(|a| a == "--headless"); // skip rendering, stats only
    let no_audio    = headless || args.iter().any(|a| a == "--no-audio"); // skip audio synthesis
    let wrap_both = args.iter().any(|a| a == "--wrap"); // --wrap enables both axes
    let wrap_x    = wrap_both || args.iter().any(|a| a == "--wrap-x");
    let wrap_y    = wrap_both || args.iter().any(|a| a == "--wrap-y");
    let bounce_x  = args.iter().any(|a| a == "--bounce-x");
    let bounce_y  = args.iter().any(|a| a == "--bounce-y");
    let steer  = args.iter().any(|a| a == "--steer");   // default: off
    let pos_rotation_enabled = !args.iter().any(|a| a == "--no-pos-color"); // default: on
    let pos_rotation_output  =  args.iter().any(|a| a == "--pos-color-out"); // default: off
    let tile_2x2             =  args.iter().any(|a| a == "--tile-2x2");      // default: off
    let dampen_x: f32 = parse_arg("--dampen-x").and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let dampen_y: f32 = parse_arg("--dampen-y").and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let rng_seed: u64 = parse_arg("--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(44);
    // --seed-density: random cells as 1/N of empty cells (0 = none)
    // Default 128 = 1/128 of empty cells
    let seed_density_inv: usize = parse_arg("--seed-density")
        .and_then(|s| s.parse().ok())
        .unwrap_or(128);

    let total_frames = seconds * FPS as usize;
    let n_chunks = (total_frames + CHUNK_FRAMES - 1) / CHUNK_FRAMES;

    println!("gravity: {}s × {}fps = {} frames, {} chunks of {} frames",
        seconds, FPS, total_frames, n_chunks, CHUNK_FRAMES);

    // ── GOOD SETTINGS (local optimum, Feb 21 2026) ───────────────────────────
    // These defaults produce genuinely interesting dynamics: Conway-active clusters
    // that move, interact, and sustain themselves without collapsing into a static blob.
    // Key properties: eff_spd≈2.2, moved≈68-75%, p10≈2.4, spread≈50-70.
    // Recommended invocation: --wrap --dampen (not default-able as boolean flags).
    // Starting point for all future exploration; escape local optima by trying
    // radically different G, softening, or init_vel — but return here if lost.
    //   G=0.075  soft=3  cap=2  pop=768  band=256  rate=16  vel=swirl  wrap  dampen
    // ─────────────────────────────────────────────────────────────────────────────

    // Sim parameters
    let g: f32 = parse_arg("--gravity")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.125_f32);
    let softening: f32 = parse_arg("--softening")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3.0_f32);
    let speed_cap: f32 = parse_arg("--speed-cap")
        .and_then(|v| v.parse().ok()).unwrap_or(2.0); // cells/frame
    let target_pop_default = 768_usize;
    let target_pop: usize = parse_arg("--pop-target")
        .and_then(|s| s.parse().ok())
        .unwrap_or(target_pop_default);
    let pop_band: f32 = parse_arg("--pop-band")
        .and_then(|s| s.parse().ok())
        .unwrap_or(256.0);
    let rate_limit: usize = parse_arg("--rate-limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(pop_band as usize); // default: same as pop_band so Conway can move pop by its full range per tick
    let conway_every: usize = parse_arg("--conway-every")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1); // default: every tick

    // --init-vel MODE: initial velocity field for seeded cells.
    //   swirl     (default) — asymmetric quadrant bias, net angular momentum
    //   random    — isotropic random ±0.25, no directional bias
    //   spin      — clockwise tangential field proportional to distance from centre; vx scaled by aspect ratio
    //   spin-flat — spin + vx*aspect + vy*0.09375 (very flat elliptical orbits, 8× flatter than original)
    //   spin-ccw  — counter-clockwise spin
    //   zero      — all seeded cells start stationary (pure gravity collapse from rest)
    let init_vel: String = parse_arg("--init-vel")
        .unwrap_or_else(|| "swirl".to_string());

    // --vel-scale F: multiply all initial velocities by F (default 1.0).
    let vel_scale: f32 = parse_arg("--vel-scale")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0_f32);

    // --circles N: place N filled disks instead of random scatter.
    // Each disk gets target_pop/N cells; radius derived from cell count.
    // Disk centres maximise min-distance from walls and each other.
    let circles: usize = parse_arg("--circles")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // --commit HASH: git commit ID for reproducibility logging (passed by run-loop.sh)
    let commit_id: String = parse_arg("--commit")
        .unwrap_or_else(|| "unknown".to_string());

    let run_id: String = parse_arg("--run-id")
        .unwrap_or_else(|| format!("seed{}", rng_seed));

    // Per-run directory: all data for this run lives under runs/{run_id}/
    let run_dir         = format!("runs/{}", run_id);
    let checkpoint_path = format!("{}/checkpoint.bin", run_dir);
    let segments_dir    = format!("{}/segments", run_dir);
    let frames_dir      = format!("{}/frames", run_dir);
    let segments_file   = format!("{}/segments.txt", run_dir);
    let shared_dir = std::env::var("GRAVITY_SHARED_DIR")
        .unwrap_or_else(|_| String::from("/Users/matte/.openclaw/workspace/shared/gravity"));
    fs::create_dir_all(&shared_dir).ok();
    // Final video alongside the run dir (runs/{run_id}.mp4) + copy to shared
    let output_file_local  = format!("runs/{}.mp4", run_id);
    let output_file_shared = format!("{}/{}.mp4", shared_dir, run_id);
    // Use local as primary; copy to shared after concat
    let output_file = output_file_local.clone();

    // Write settings file alongside video and run_info for the watcher
    let init_pop = if circles > 0 {
        // ~50% coin-flip density over disk area → expected placed ≈ target_pop
        target_pop
    } else if seed_density_inv > 0 { W * H / seed_density_inv } else { 0 };
    let circles_str = if circles > 0 { format!("{}", circles) } else { "none".to_string() };
    let settings = format!(
        "run_id:        {run_id}\nseed:          {rng_seed}\nseconds:       {seconds}\n\
         commit:        {commit_id}\n\
         gravity:       {g}\nsoftening:     {softening}\nspeed_cap:     {speed_cap}\n\
         pop_target:    {target_pop}\npop_band:      {pop_band}\nrate_limit:    {rate_limit}\nconway_every:  {conway_every}\n\
         seed_density:  1/{seed_density_inv}\ninit_pop:      {init_pop}\ninit_vel:      {init_vel}\n\
         circles:       {circles_str}\nvel_scale:     {vel_scale}\n\
         wrap_x:        {wrap_x}\nwrap_y:        {wrap_y}\nbounce_x:      {bounce_x}\nbounce_y:      {bounce_y}\ndampen_x:      {dampen_x}\ndampen_y:      {dampen_y}\nsteer:         {steer}\n\
         pos_color_in:  {pos_rotation_enabled}\npos_color_out: {pos_rotation_output}\ntile_2x2:      {tile_2x2}\n\
         resolution:    {}x{} → 2048x1280\n",
        OUT_W * 2, OUT_H * 2
    );
    fs::create_dir_all(&segments_dir).unwrap();
    fs::create_dir_all(&frames_dir).unwrap();
    fs::create_dir_all("runs").unwrap();
    // Write run_info to the run dir AND to state/ (watcher compat pointer)
    let run_info_content = format!("run_id={run_id}\n{settings}");
    let _ = fs::write(format!("{}/run_info.txt", run_dir), &run_info_content);
    fs::create_dir_all("state").unwrap();
    let _ = fs::write("state/run_info.txt", &run_info_content);
    // Settings copy to shared dir for reference
    let _ = fs::write(format!("{}/{}.txt", shared_dir, run_id), &settings);

    // Load checkpoint or init fresh
    let (mut sim, mut canvas, start_chunk) =
        Sim::load_checkpoint(&checkpoint_path, g, softening, speed_cap, pop_band, rate_limit, conway_every, seed_density_inv, target_pop, wrap_x, wrap_y, bounce_x, bounce_y, steer, dampen_x, dampen_y)
        .map(|(s, c, ci)| {
            println!("Resuming from checkpoint: chunk {}/{}", ci, n_chunks);
            (s, c, ci)
        })
        .unwrap_or_else(|| {
            if circles > 0 {
                println!("Fresh start [{run_id}] seed={rng_seed} circles={circles}");
            } else {
                println!("Fresh start [{run_id}] seed={rng_seed} density=1/{seed_density_inv}");
            }
            let s = Sim::new(rng_seed, g, softening, speed_cap, pop_band, rate_limit, conway_every, seed_density_inv, target_pop, wrap_x, wrap_y, bounce_x, bounce_y, steer, dampen_x, dampen_y, &init_vel, circles, vel_scale);
            let c = vec![0.0f32; W * H * 3];
            (s, c, 0)
        });

    let orig_state_path = format!("{}/orig_state.bin", run_dir);

    // OriginalState = tick=0 layout. On fresh start: capture now and persist.
    // On checkpoint resume: load from disk so epilogue targets the actual first frame.
    let orig = if start_chunk == 0 {
        // Fresh start — this IS tick=0
        let o = OriginalState {
            positions:  sim.cells.iter().map(|c| (c.x as usize % W, c.y as usize % H)).collect(),
            velocities: sim.cells.iter().map(|c| ((c.x as usize % W, c.y as usize % H), (c.vx, c.vy))).collect(),
            count: sim.cells.len(),
        };
        Sim::save_orig_state(&o, &orig_state_path);
        println!("Saved original state ({} cells) for epilogue target.", o.count);
        o
    } else {
        // Checkpoint resume — load the tick=0 state saved on fresh start
        match Sim::load_orig_state(&orig_state_path) {
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
        .open(&segments_file).unwrap();

    'chunks: for chunk in start_chunk..n_chunks {
        let chunk_start_frame = chunk * CHUNK_FRAMES;
        let chunk_end_frame = ((chunk + 1) * CHUNK_FRAMES).min(total_frames);
        let this_chunk_frames = chunk_end_frame - chunk_start_frame;

        println!("\n[chunk {}/{n_chunks}] frames {}..{}", chunk+1, chunk_start_frame, chunk_end_frame);
        let mut chunk_audio: Vec<f32> = Vec::with_capacity(SAMPLES_PER_FRAME * this_chunk_frames * 2); // stereo interleaved

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
            if !headless {
                sim.paint_frame(&mut canvas);
                Sim::save_png(&canvas, &format!("{frames_dir}/f{global_frame:08}.png"));
            }
            sim.tick();
            if !no_audio { sim.generate_audio(&mut chunk_audio); }

            let log_every = if headless { FPS as usize } else { 480 };
            if local_frame % log_every == 0 {
                println!("  frame {}/{total_frames}  {}", global_frame, sim.stats());
            }
        }

        if !headless {
            // Encode chunk
            let seg_path = format!("{segments_dir}/seg_{chunk_start_frame:08}.mp4");
            encode_chunk(frames_dir, &seg_path, this_chunk_frames);

            // Append to segments list
            writeln!(seg_list, "file '{seg_path}'").unwrap();
            seg_list.flush().unwrap();

            // Delete PNGs
            delete_frames(frames_dir);
        }

        // Save checkpoint (next chunk index)
        sim.save_checkpoint(&canvas, chunk + 1, &checkpoint_path);

        let pct = (chunk + 1) * 100 / n_chunks;
        let pop = sim.cells.len();
        println!("  chunk {}/{n_chunks} done ({pct}%)  pop={pop}", chunk+1);
        // Write stats for segment-watcher.sh to include in Discord messages
        let _ = fs::write("state/last_stats.txt", format!("pop={pop}\ntarget=2560\nrange=[1920,3200]\n"));
    }

    // ── Epilogue phase ────────────────────────────────────────────────────
    if do_epilogue {
        let palette = load_palette();
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
            let fade = 0.999068_f32.powf(1.0 - t) * 0.5_f32.powf(t * 0.25);
            // Only fade L (brightness); a and b are irrelevant as L→0
            for px in canvas.chunks_exact_mut(3) { px[0] = (px[0] * fade).max(0.0); }
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
    let total_segs = fs::read_to_string(&segments_file).unwrap_or_default().lines().count();
    println!("\nConcatenating {total_segs} segments → {output_file}");
    concat_segments(&segments_file, &output_file);

    let size = fs::metadata(&output_file).map(|m| m.len()).unwrap_or(0);
    println!("Done! {output_file} ({:.1} MB)", size as f64 / 1_048_576.0);

    // Clean up checkpoint on successful completion
    let _ = fs::remove_file(checkpoint_path);
    println!("Checkpoint removed.");
}

// [recovery] edit target not found, appending:
use std::io::{BufWriter, Write};

// [recovery] edit target not found, appending:
        // Enforce per-component: clamp births and deaths independently.
        // Births can't push us above pop_max; deaths can't push us below pop_min.
        // Additionally rate-limit to ceil(pop_band/2) per tick — 1/4 of total band range.
        // This applies even when outside the band, preventing runaway explosions/crashes.
        let rate_limit = ((self.pop_band / 2.0).ceil() as usize).max(1);
        let max_births = pop_max.saturating_sub(n).min(rate_limit);
        let max_deaths = n.saturating_sub(pop_min).min(rate_limit);
        desired_births.truncate(max_births);
        desired_deaths.truncate(max_deaths);

// [recovery] edit target not found, appending:
    fn gravity_step_epilogue(&mut self, orig: &OriginalState, g_scale: f32) {
        let n = self.cells.len();
        // Barnes-Hut for epilogue: original particles are fixed, no force applied to them
        let mut nodes: Vec<QNode> = Vec::with_capacity(n * 8);
        nodes.push(QNode::empty(0.0, 0.0, W as f32, H as f32));
        for i in 0..n {
            let (px, py) = (self.cells[i].x, self.cells[i].y);
            qt_insert(&mut nodes, 0, i, px, py, 0);
        }
        for i in 0..n {
            let is_orig = orig.positions.contains(
                &(self.cells[i].x as usize % W, self.cells[i].y as usize % H));
            if is_orig { continue; }
            let (px, py) = (self.cells[i].x, self.cells[i].y);
            let (fx, fy) = qt_force(&nodes, 0, i, px, py, self.g * g_scale, self.softening);

// [recovery] edit target not found, appending:
        // Rate-limit: 1 birth and 1 death per Conway call (independent of pop_band).
        // With conway_every=FPS this equals 1 per second.
        let rate_limit = 1_usize;

// [recovery] edit target not found, appending:
    fn gravity_step_epilogue(&mut self, orig: &OriginalState, g_scale: f32) {
        let n = self.cells.len();
        // Barnes-Hut for epilogue: forces only, no movement — positions stay integer-discrete.
        // Original-position cells are frozen (no force applied); non-originals get force
        // but movement is handled by the kill/revive nudges, not direct position update.
        let mut nodes: Vec<QNode> = Vec::with_capacity(n * 8);
        nodes.push(QNode::empty(0.0, 0.0, W as f32, H as f32));
        for i in 0..n {
            let (px, py) = (self.cells[i].x as f32 + 0.5, self.cells[i].y as f32 + 0.5);
            qt_insert(&mut nodes, 0, i, px, py, 0);
        }
        for i in 0..n {
            if orig.positions.contains(&(self.cells[i].gx(), self.cells[i].gy())) { continue; }
            let (px, py) = (self.cells[i].px, self.cells[i].py);
            let (gfx, gfy) = qt_force(&nodes, 0, i, px, py, self.g * g_scale, self.softening);
            self.cells[i].vx += gfx;
            self.cells[i].vy += gfy;
        }
        // Cap speeds (velocity still evolves, even though positions don't move this phase)
        for c in &mut self.cells {
            let spd = (c.vx*c.vx+c.vy*c.vy).sqrt();
            let cap = c.prev_speed.max(self.speed_cap);
            if spd > cap { c.vx = c.vx/spd*cap; c.vy = c.vy/spd*cap; }
            let hard_ceil = self.speed_cap * 2.0;
            c.prev_speed = c.prev_speed.min(spd).max(self.speed_cap).min(hard_ceil);
        }
        self.order = (0..self.cells.len()).collect();
    }

// [recovery] edit target not found, appending:
        let neighbour_offsets: [(i32, i32); 8] = [
            (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
        ];
        let (wrap_x, wrap_y) = (self.wrap_x, self.wrap_y);
        // Resolve a neighbour offset to a grid index, respecting per-axis wrap.
        let resolve_nbr = |gy: usize, gx: usize, dy: i32, dx: i32| -> Option<(usize, usize)> {
            let ry = gy as i32 + dy;
            let rx = gx as i32 + dx;
            let ry = if wrap_y { Some(ry.rem_euclid(H as i32) as usize) }
                     else if ry >= 0 && ry < H as i32 { Some(ry as usize) }
                     else { None };
            let rx = if wrap_x { Some(rx.rem_euclid(W as i32) as usize) }
                     else if rx >= 0 && rx < W as i32 { Some(rx as usize) }
                     else { None };
            match (ry, rx) { (Some(ry), Some(rx)) => Some((ry, rx)), _ => None }
        };
        let live_neighbours = |gy: usize, gx: usize| -> Vec<usize> {
            neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let (ny, nx) = resolve_nbr(gy, gx, dy, dx)?;
                let idx = grid[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect()
        };

        let mut desired_births: Vec<(usize, usize, Vec<usize>)> = Vec::new();
        let mut desired_deaths: Vec<usize> = Vec::new();

        for (i, c) in self.cells.iter().enumerate() {
            let gx = c.x;
            let gy = c.y;
            let nbrs = live_neighbours(gy, gx);
            let count = nbrs.len();
            if count != 2 && count != 3 {
                if !nbrs.is_empty() { desired_deaths.push(i); }
            }
        }

        let mut candidates = std::collections::HashSet::new();
        for c in &self.cells {
            let gx = c.x;
            let gy = c.y;
            for &(dy, dx) in &neighbour_offsets {
                if let Some((ny, nx)) = resolve_nbr(gy, gx, dy, dx) {
                    if grid[ny * W + nx] == usize::MAX { candidates.insert((ny, nx)); }
                }
            }
        }

// [recovery] edit target not found, appending:
            let live_nbrs: Vec<usize> = neighbour_offsets.iter().filter_map(|&(dy, dx)| {
                let (ny, nx) = resolve_nbr(gy, gx, dy, dx)?;
                let idx = grid2[ny * W + nx];
                if idx != usize::MAX { Some(idx) } else { None }
            }).collect();
            if live_nbrs.is_empty() { continue; }
            let (vx, vy) = if self.wrap_x || self.wrap_y {
                // Wrap mode: inherit avg neighbour velocity for interesting dynamics
                let n_nbrs = live_nbrs.len() as f32;
                let vx = live_nbrs.iter().map(|&i| self.cells[i].vx).sum::<f32>() / n_nbrs;
                let vy = live_nbrs.iter().map(|&i| self.cells[i].vy).sum::<f32>() / n_nbrs;
                (vx, vy)
            } else {
                // No-wrap: born at rest — gravity provides velocity organically.
                // Inheriting neighbour velocity near walls continuously injects wall-facing
                // momentum faster than gravity can correct it.
                (0.0_f32, 0.0_f32)
            };
            let birth_spd = (vx * vx + vy * vy).sqrt();
            let new_idx = self.cells.len();
            self.cells.push(Cell { px: gx as f32 + 0.5, py: gy as f32 + 0.5, vx, vy, prev_speed: birth_spd });
            grid2[gy * W + gx] = new_idx;

// [recovery] edit target not found, appending:
            let new_idx = self.cells.len();
            self.cells.push(Cell { x: gx, y: gy, vx, vy, prev_speed: birth_spd });
            grid2[gy * W + gx] = new_idx;
            self.conway_births += 1;
        }

        self.order = (0..self.cells.len()).collect();
    }

    // ── Gravity step (Barnes-Hut O(n log n)) ──────────────────────────────

// [recovery] edit target not found, appending:
        let o = OriginalState {
            positions:  sim.cells.iter().map(|c| (c.gx(), c.gy())).collect(),
            velocities: sim.cells.iter().map(|c| ((c.gx(), c.gy()), (c.vx, c.vy))).collect(),
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
                    positions:  sim.cells.iter().map(|c| (c.gx(), c.gy())).collect(),
                    velocities: sim.cells.iter().map(|c| ((c.gx(), c.gy()), (c.vx, c.vy))).collect(),
                    count: sim.cells.len(),
                }
            }
        }
    };

// [recovery] edit target not found, appending:
        // Movement: float positions, collision by grid square.
        // Process in shuffled order. Each cell computes its target float position (px+vx, py+vy).
        // If the target grid square is free: move (update both float pos and grid).
        // If occupied or same square: stay put entirely — no float accumulation.
        // In no-wrap mode: if target would be out of bounds, stay put — gravity must pull back.
        let mut grid = vec![usize::MAX; W * H];
        for (i, c) in self.cells.iter().enumerate() {
            grid[c.gy() * W + c.gx()] = i;
        }
        let n = self.order.len();
        for i in (1..n).rev() {
            let j = (xoru64(&mut self.rng) as usize) % (i + 1);
            self.order.swap(i, j);
        }
        for &idx in &self.order {
            // Capture pre-move state for audio stats (avoid borrow conflict later)
            let (cvx, cvy, cpx, cpy) = {
                let c = &self.cells[idx]; (c.vx, c.vy, c.px, c.py)
            };
            let old_gx = ((cpx.floor() as i32).rem_euclid(W as i32)) as usize;
            let old_gy = ((cpy.floor() as i32).rem_euclid(H as i32)) as usize;
            // Resolve position per axis: wrap / bounce / hard-wall
            let mut nx = cpx + cvx;
            let mut ny = cpy + cvy;
            let mut nvx = cvx;
            let mut nvy = cvy;
            // X axis
            if self.wrap_x {
                nx = nx.rem_euclid(W as f32);
            } else if self.bounce_x {
                if nx < 0.0       { nx = -nx;                      nvx = -nvx; }
                else if nx >= W as f32 { nx = 2.0 * W as f32 - nx; nvx = -nvx; }
            } else if nx < 0.0 || nx >= W as f32 { continue; }
            // Y axis
            if self.wrap_y {
                ny = ny.rem_euclid(H as f32);
            } else if self.bounce_y {
                if ny < 0.0       { ny = -ny;                      nvy = -nvy; }
                else if ny >= H as f32 { ny = 2.0 * H as f32 - ny; nvy = -nvy; }
            } else if ny < 0.0 || ny >= H as f32 { continue; }
            // Apply any velocity changes from bounce before grid logic
            if nvx != cvx { self.cells[idx].vx = nvx; }
            if nvy != cvy { self.cells[idx].vy = nvy; }
            let (new_px, new_py) = (nx, ny);
            let tgx = (new_px.floor() as i32).rem_euclid(W as i32) as usize;
            let tgy = (new_py.floor() as i32).rem_euclid(H as i32) as usize;
            let crossing = tgx != old_gx || tgy != old_gy;
            let moved_cross;
            if !crossing {
                // Same grid square — update float position freely
                self.cells[idx].px = new_px;
                self.cells[idx].py = new_py;
                moved_cross = false;
            } else if grid[tgy * W + tgx] == usize::MAX {
                // Target square free — move
                grid[old_gy * W + old_gx] = usize::MAX;
                grid[tgy * W + tgx] = idx;
                self.cells[idx].px = new_px;
                self.cells[idx].py = new_py;
                self.cells[idx].moved = true;
                moved_cross = true;
            } else {
                // Target occupied — stay put
                moved_cross = false;
            }
        }

        // ── Audio: accumulate spatial region stats (all cells, post-move) ──────
        // Divide canvas into 3×3 regions. Each cell contributes to its region's
        // population count, total speed, and CoG sum.
        for c in &self.cells {
            let col = ((c.px / W as f32) * 3.0).floor().clamp(0.0, 2.0) as usize;
            let row = ((c.py / H as f32) * 3.0).floor().clamp(0.0, 2.0) as usize;
            let ri = row * 3 + col;
            let speed = (c.vx * c.vx + c.vy * c.vy).sqrt();
            let rs = &mut self.region_stats[ri];
            rs.cell_count += 1.0;
            rs.speed_sum  += speed;
            rs.cog_x_sum  += c.px;
            rs.cog_y_sum  += c.py;
        }
    }

// [recovery] edit target not found, appending:
            let gx = self.cells[di].gx();
            let gy = self.cells[di].gy();

// [recovery] edit target not found, appending:
            self.cells.push(Cell { px: gx as f32 + 0.5, py: gy as f32 + 0.5, vx, vy, prev_speed: spd });

// [recovery] edit target not found, appending:
        // Build quadtree with a SQUARE root centered on the grid center.
        // The grid is W×H = 192×120 (non-square). A non-square root means
        // node.width() = max(x_range, y_range) always equals the x dimension,
        // making the BH opening criterion systematically less accurate for y forces.
        // A square root at size max(W,H) makes every sub-node square, so the
        // criterion is identical for x and y — no directional bias.
        let mut nodes: Vec<QNode> = Vec::with_capacity(n * 8);
        {
            let half = 128.0_f32; // 256×256 square, power-of-2 subdivisions
            let cx = W as f32 * 0.5; // 96
            let cy = H as f32 * 0.5; // 60
            // Root: [-32, 224] × [-68, 188] — 256×256, centred on grid centre
            nodes.push(QNode::empty(cx - half, cy - half, cx + half, cy + half));
        }
        for i in 0..n {
            let (px, py) = (self.cells[i].px, self.cells[i].py);
            qt_insert(&mut nodes, 0, i, px, py, 0);
        }

// [recovery] edit target not found, appending:
    fn stats(&self) -> String {
        let total = self.cells.len() as f32;
        if total == 0.0 {
            return "pop=0 births=0 deaths=0 avg_spd=0 max=0 p10=0 spread=0 blk=0/0 com=(0,0)".into();
        }
        let in_bounds: Vec<&Cell> = if self.wrap_x && self.wrap_y {
            self.cells.iter().collect()
        } else {
            self.cells.iter().filter(|c| c.in_bounds()).collect()
        };
        let pop = in_bounds.len();
        let n = total;
        let cx = self.cells.iter().map(|c| c.px).sum::<f32>() / n;
        let cy = self.cells.iter().map(|c| c.py).sum::<f32>() / n;
        let spread = self.cells.iter().map(|c| {
            let dx = c.px - cx; let dy = c.py - cy;
            (dx*dx+dy*dy).sqrt()
        }).sum::<f32>() / n;

        // Speed stats: avg, max, p10 — stuck cells count at 1/128 speed (hint, not zero).
        let mut speeds: Vec<f32> = self.cells.iter()
            .map(|c| { let s = (c.vx*c.vx+c.vy*c.vy).sqrt(); if c.moved { s } else { s / 128.0 } })
            .collect();
        speeds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg_spd = speeds.iter().sum::<f32>() / n;
        let max_spd = *speeds.last().unwrap_or(&0.0);
        let p10_idx = ((speeds.len() as f32 * 0.10) as usize).min(speeds.len().saturating_sub(1));
        let p10_spd = speeds[p10_idx];

        // Effective speed: cells that didn't actually move last tick count as 0.
        // Reveals true visual motion — packed cells have velocity but are frozen in place.
        let eff_spd = self.cells.iter()
            .map(|c| if c.moved { (c.vx*c.vx+c.vy*c.vy).sqrt() } else { 0.0 })
            .sum::<f32>() / n;
        let moved_frac = self.cells.iter().filter(|c| c.moved).count() as f32 / n;

        // Clustering: divide grid into BLK×BLK blocks, count occupied blocks
        // Low blk = tight clusters; high blk = spread across grid
        const BLK: usize = 8;
        const BROWS: usize = (H + BLK - 1) / BLK;  // 20
        const BCOLS: usize = (W + BLK - 1) / BLK;  // 32
        const BTOTAL: usize = BROWS * BCOLS;          // 640
        let mut block_occ = [false; BTOTAL];
        let mut max_in_block = 0u16;
        let mut block_counts = [0u16; BTOTAL];
        for c in &self.cells {
            let bx = (c.px as usize / BLK).min(BCOLS - 1);
            let by = (c.py as usize / BLK).min(BROWS - 1);
            let bi = by * BCOLS + bx;
            block_occ[bi] = true;
            block_counts[bi] += 1;
            if block_counts[bi] > max_in_block { max_in_block = block_counts[bi]; }
        }
        let blk_used = block_occ.iter().filter(|&&v| v).count();

        // Top-3 densest block coordinates — track these across samples to detect blob drift
        let mut block_list: Vec<(u16, usize, usize)> = block_counts.iter().enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(bi, &c)| (c, bi % BCOLS, bi / BCOLS))
            .collect();
        block_list.sort_by(|a, b| b.0.cmp(&a.0));
        let hot: String = block_list.iter().take(3)
            .map(|(_, bx, by)| format!("({},{})", bx * BLK, by * BLK))
            .collect::<Vec<_>>().join(";");

        format!("pop={pop} births={} deaths={} avg_spd={avg_spd:.3} eff_spd={eff_spd:.3} moved={moved_pct:.0}% max={max_spd:.3} p10={p10_spd:.3} spread={spread:.1} blk={blk_used}/{BTOTAL} dense={max_in_block} hot=[{hot}] com=({cx:.1},{cy:.1})",
            self.conway_births, self.conway_deaths,
            moved_pct = moved_frac * 100.0)
    }

// [recovery] edit target not found, appending:
                let spd   = xorf32(&mut rng) * speed_cap * 0.001; // tiny perturbation — gravity does the work

// [recovery] edit target not found, appending:
                // Tiny random initial velocity: speed ~ U[0, 0.003% of speed_cap], gravity does the work
                let spd   = xorf32(&mut rng) * speed_cap * 0.00003125; // 0.001 / 32

// [recovery] edit target not found, appending:
            if !occupied[idx] {
                let cx = W as f32 / 2.0;
                let cy = H as f32 / 2.0;
                let (vx, vy) = match init_vel {
                    "swirl" => {
                        // Asymmetric quadrant bias — creates net angular momentum.
                        // Top-left biased right, bottom-right biased left,
                        // bottom-left biased down, top-right unbiased.
                        if xi < W / 2 && yi < H / 2 {
                            (xorf32(&mut rng) * 0.75 - 0.25, (xorf32(&mut rng) - 0.5) * 0.25)
                        } else if xi >= W / 2 && yi >= H / 2 {
                            (xorf32(&mut rng) * 0.75 - 0.5,  (xorf32(&mut rng) - 0.5) * 0.25)
                        } else if xi >= W / 2 {
                            ((xorf32(&mut rng) - 0.5) * 0.25, (xorf32(&mut rng) - 0.5) * 0.25)
                        } else {
                            ((xorf32(&mut rng) - 0.5) * 0.25, xorf32(&mut rng) * 0.25)
                        }
                    }
                    "random" => {
                        // Isotropic random — no net angular momentum or linear drift.
                        ((xorf32(&mut rng) - 0.5) * 0.5, (xorf32(&mut rng) - 0.5) * 0.5)
                    }
                    "spin" => {
                        // Clockwise tangential velocity field.
                        // Speed proportional to distance from centre, capped at 0.5.
                        let dx = xi as f32 + 0.5 - cx;
                        let dy = yi as f32 + 0.5 - cy;
                        let r = (dx * dx + dy * dy).sqrt().max(1.0);
                        let scale = (r / (cx.min(cy))).min(1.0) * 0.5;
                        // Clockwise tangent: (-dy/r, dx/r)
                        let noise_x = (xorf32(&mut rng) - 0.5) * 0.1;
                        let noise_y = (xorf32(&mut rng) - 0.5) * 0.1;
                        (-dy / r * scale + noise_x, dx / r * scale + noise_y)
                    }
                    "spin-ccw" => {
                        // Counter-clockwise tangential velocity field.
                        let dx = xi as f32 + 0.5 - cx;
                        let dy = yi as f32 + 0.5 - cy;
                        let r = (dx * dx + dy * dy).sqrt().max(1.0);
                        let scale = (r / (cx.min(cy))).min(1.0) * 0.5;
                        let noise_x = (xorf32(&mut rng) - 0.5) * 0.1;
                        let noise_y = (xorf32(&mut rng) - 0.5) * 0.1;
                        (dy / r * scale + noise_x, -dx / r * scale + noise_y)
                    }
                    "radial-out" => {
                        // Radially outward from centre — dramatic infall after reversal.
                        let dx = xi as f32 + 0.5 - cx;
                        let dy = yi as f32 + 0.5 - cy;
                        let r = (dx * dx + dy * dy).sqrt().max(1.0);
                        let scale = 0.4;
                        let noise_x = (xorf32(&mut rng) - 0.5) * 0.1;
                        let noise_y = (xorf32(&mut rng) - 0.5) * 0.1;
                        (dx / r * scale + noise_x, dy / r * scale + noise_y)
                    }
                    "zero" | _ => {
                        // All seeded cells start stationary — pure gravity collapse from rest.
                        (0.0, 0.0)
                    }
                };
                cells.push(Cell { px: xi as f32 + 0.5, py: yi as f32 + 0.5, vx, vy, prev_speed: 0.0 });

// [recovery] edit target not found, appending:
        // Momentum damping: remove dampen_x/dampen_y fraction of COM velocity each tick.
        // e.g. dampen_y=0.125 removes 12.5% of avg vertical velocity per tick.
        if (self.dampen_x > 0.0 || self.dampen_y > 0.0) && !self.cells.is_empty() {
            let n = self.cells.len() as f32;
            let avg_vx = self.cells.iter().map(|c| c.vx).sum::<f32>() / n;
            let avg_vy = self.cells.iter().map(|c| c.vy).sum::<f32>() / n;
            for c in &mut self.cells {
                c.vx -= avg_vx * self.dampen_x;
                c.vy -= avg_vy * self.dampen_y;
            }
        }

// [recovery] edit target not found, appending:
        let n = cells.len();
        let target_pop = W * H / 16;
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap, start_pop: target_pop,

// [recovery] edit target not found, appending:
        let target_pop = W * H / 16;
                        start_pop: target_pop, pop_band, rate_limit,

// [recovery] edit target not found, appending:
        let order = (0..cells.len()).collect();
        let sim = Sim { cells, order, rng, g, softening, speed_cap,

// [recovery] edit target not found, appending:
    // ── Audio synthesis ────────────────────────────────────────────────────
    // Called once per video frame. Appends SAMPLES_PER_FRAME f32 samples to chunk_audio.
    // Only cells that moved (changed grid square) this tick sustain a voice.
    // Stationary/blocked cells let their voice release.
    fn generate_audio(&mut self, chunk_audio: &mut Vec<f32>) {
        use std::f32::consts::PI;

        // ── 1. Derive targets from accumulated region stats ────────────────
        // Region dimensions in world units
        let rw = W as f32 / 3.0; // width of one region column
        let rh = H as f32 / 3.0; // height of one region row

        for (ri, v) in self.region_voices.iter_mut().enumerate() {
            let rs = &self.region_stats[ri];
            let col = (ri % 3) as f32;
            let row = (ri / 3) as f32;

            // Amplitude target: population × avg_speed, normalised
            let amp_target = if rs.cell_count > 0.0 {
                let avg_speed = rs.speed_sum / rs.cell_count;
                // Scale: ~100 cells × speed 2.0 → amplitude 1.0
                (rs.cell_count * avg_speed / 200.0).min(2.0)
            } else {
                0.0
            };
            v.amplitude += (amp_target - v.amplitude) * SLEW_AMP;

            // CoG — normalised within the region [0..1], defaulting to centre when empty
            let (cog_x_target, cog_y_target) = if rs.cell_count > 0.0 {
                let cx = (rs.cog_x_sum / rs.cell_count - col * rw) / rw;
                let cy = (rs.cog_y_sum / rs.cell_count - row * rh) / rh;
                (cx.clamp(0.0, 1.0), cy.clamp(0.0, 1.0))
            } else {
                (0.5, 0.5) // drift toward centre when idle
            };
            v.cog_x += (cog_x_target - v.cog_x) * SLEW_COG;
            v.cog_y += (cog_y_target - v.cog_y) * SLEW_COG;

            // Pitch bend: CoG-x drives ±PITCH_BEND_MAX cents
            let bend_target = (v.cog_x - 0.5) * 2.0 * PITCH_BEND_MAX;
            v.pitch_bend += (bend_target - v.pitch_bend) * SLEW_BEND;

            // Filter: CoG-y drives warmth — top of region (cog_y→0) = bright, bottom = warm
            let filter_target = FILTER_BRIGHT + (FILTER_WARM - FILTER_BRIGHT) * v.cog_y;
            v.filter_coeff += (filter_target - v.filter_coeff) * SLEW_FILTER;
        }

        // ── 2. Synthesise SAMPLES_PER_FRAME stereo pairs ──────────────────
        for _ in 0..SAMPLES_PER_FRAME {
            let mut sum_l     = 0.0f32;
            let mut sum_r     = 0.0f32;
            let mut reverb_in = 0.0f32;

            for v in self.region_voices.iter_mut() {
                // Hard gate: silence very quiet voices to prevent droning
                if v.amplitude < 1e-4 { continue; }

                // Pitch with CoG-x bend
                let freq = v.base_freq * 2.0f32.powf(v.pitch_bend / 1200.0);
                v.phase = (v.phase + freq / SAMPLE_RATE as f32).rem_euclid(1.0);

                // Sine wave through dynamic LP filter (CoG-y driven warmth)
                let raw = (v.phase * 2.0 * PI).sin();
                v.filter_state += v.filter_coeff * (raw - v.filter_state);

                let s = v.filter_state * v.amplitude * AUDIO_AMP_SCALE;

                // Equal-power pan (fixed per column)
                let pan_angle = (v.pan + 1.0) * 0.5 * PI * 0.5;
                sum_l     += s * pan_angle.cos();
                sum_r     += s * pan_angle.sin();
                reverb_in += s * v.reverb_send;
            }

            // Mid-side reverb
            let wet  = self.reverb.process(reverb_in.tanh() * 0.7);
            let side = (sum_l - sum_r) * 0.5;
            chunk_audio.push((wet + side).tanh()); // L
            chunk_audio.push((wet - side).tanh()); // R
        }

        // ── 3. Clear stats for next frame ──────────────────────────────────
        self.region_stats = [RegionStats::default(); 9];
    }

    fn paint_frame(&mut self, canvas: &mut Vec<f32>) {
        // Canvas stores Oklab (L, a, b) as f32 per channel.
        // Fade only L (brightness): multiplicative + constant drain so L always reaches 0.
        // a and b (chroma) are left intact — they become invisible as L→0.
        const FADE_SLOW: f32 = 0.999068; // 0.999534² — doubled fade speed
        // Epsilon ensures L hits 0 within ~28s at 60fps (not stuck at grey asymptote).
        // At FADE_SLOW, without epsilon, a cell starting at L=0.75 would asymptote to ~0.32.
        const FADE_EPSILON: f32 = 0.0003;
        for py in 0..H {
            for px in 0..W {
                let i = (py * W + px) * 3;
                if self.prev_live[py * W + px] {
                    // Was alive last tick, now gone — fast brightness drop (trail burst)
                    canvas[i] = (canvas[i] * 0.5).max(0.0);
                    // a, b unchanged
                } else {
                    // Normal background fade — only L drained
                    canvas[i] = (canvas[i] * FADE_SLOW - FADE_EPSILON).max(0.0);
                    // a, b unchanged
                }
            }
        }
        self.prev_live.fill(false);
        for c in &self.cells {
            let xi = c.gx();
            let yi = c.gy();
            // Stuck cells dimmed by 1/8 of their value (×0.875), not to 1/8.
            let (cvx, cvy) = if c.moved { (c.vx, c.vy) } else { (c.vx * 0.875, c.vy * 0.875) };
            let (l, a, b) = velocity_color_oklab(cvx, cvy, c.px, c.py, self.speed_cap, palette);
            let i = (yi * W + xi) * 3;
            canvas[i]     = l;
            canvas[i + 1] = a;
            canvas[i + 2] = b;
            self.prev_live[yi * W + xi] = true;
        }
    }

    fn save_png(canvas: &[f32], path: &str) {
        // Convert Oklab (L, a, b) → sRGB u8 only at output time.
        let pixels: Vec<u8> = canvas.chunks_exact(3)
            .flat_map(|px| {
                let (r, g, b) = oklab_to_srgb(px[0], px[1], px[2]);
                [r, g, b]
            })
            .collect();

// [recovery] edit target not found, appending:
            let birth_spd = (vx * vx + vy * vy).sqrt();
            let new_idx = self.cells.len();
            let id = self.next_id; self.next_id += 1;
            let (bpx, bpy) = (gx as f32 + 0.5, gy as f32 + 0.5);
            self.cells.push(Cell { px: bpx, py: bpy, vx, vy, prev_speed: birth_spd, id, moved: false });
            // (no per-event audio in new direction-bucket system)
            grid2[gy * W + gx] = new_idx;
            self.conway_births += 1;

// [recovery] edit target not found, appending:
        for &(ox, oy) in &orig.positions {
            if grid2[oy * W + ox] == usize::MAX && xorf32(&mut self.rng) < revive_chance {
                let id = self.next_id; self.next_id += 1;
                self.cells.push(Cell { px: ox as f32 + 0.5, py: oy as f32 + 0.5,
                                       vx: 0.0, vy: 0.0, prev_speed: 0.0, id, moved: false });
            }
        }

        // Lerp velocities of live-original cells 3.125% closer to their original velocity each tick (4× slower)

// [recovery] edit target not found, appending:
            let mut grid2 = vec![usize::MAX; W * H];
            for (i, c) in self.cells.iter().enumerate() {
                grid2[c.gy() * W + c.gx()] = i;
            }
            for &(ox, oy) in &orig.positions {
                if grid2[oy * W + ox] == usize::MAX && xorf32(&mut self.rng) < revive_chance {
                    let id = self.next_id; self.next_id += 1;
                    self.cells.push(Cell { px: ox as f32 + 0.5, py: oy as f32 + 0.5,
                                           vx: 0.0, vy: 0.0, prev_speed: 0.0, id, moved: false });
                }
            }
        }

// [recovery] edit target not found, appending:
            let new_idx = self.cells.len();
            let spd = (vx*vx+vy*vy).sqrt();
            let id = self.next_id; self.next_id += 1;
            self.cells.push(Cell { px: gx as f32 + 0.5, py: gy as f32 + 0.5, vx, vy, prev_speed: spd, id, moved: false });
            grid2[gy * W + gx] = new_idx;

// [recovery] edit target not found, appending:
        if !headless {
            // Encode chunk
            let seg_path = format!("{segments_dir}/seg_{chunk_start_frame:013}.mp4");
            encode_chunk(frames_dir, &seg_path, this_chunk_frames);
            mux_audio_into_segment(&seg_path, &chunk_audio);

            // Append to segments list

// [recovery] edit target not found, appending:
        println!("\n[chunk {}/{n_chunks}] frames {}..{}", chunk+1, chunk_start_frame, chunk_end_frame);
        let mut chunk_audio: Vec<f32> = Vec::with_capacity(SAMPLES_PER_FRAME * this_chunk_frames);

        // Render frames for this chunk — check signal each frame
        let sim_t0 = std::time::Instant::now();
        for local_frame in 0..this_chunk_frames {
            if !keep_running.load(Ordering::Relaxed) {
                // Discard partial chunk and stop immediately
                println!("[signal] Discarding partial chunk {}, cleaning up {} frames...",
                    chunk + 1, local_frame);
                delete_frames(frames_dir);
                break 'chunks;
            }
            let global_frame = chunk_start_frame + local_frame;
            if !headless {
                sim.paint_frame(&mut canvas);
                Sim::save_png(&canvas, &format!("{frames_dir}/f{global_frame:013}.png"));
            }
            sim.tick();
            sim.generate_audio(&mut chunk_audio);

            let log_every = if headless { FPS as usize } else { 480 };
            if local_frame % log_every == 0 {
                println!("  frame {}/{total_frames}  {}", global_frame, sim.stats());
            }
        }
        let sim_ms = sim_t0.elapsed().as_millis();

        let enc_ms;
        if !headless {
            // Encode chunk
            let enc_t0 = std::time::Instant::now();
            let seg_path = format!("{segments_dir}/seg_{chunk_start_frame:013}.mp4");
            encode_chunk(frames_dir, &seg_path, this_chunk_frames);
            mux_audio_into_segment(&seg_path, &chunk_audio);
            enc_ms = enc_t0.elapsed().as_millis();

            // Append to segments list
            writeln!(seg_list, "file '{seg_path}'").unwrap();
            seg_list.flush().unwrap();

            // Delete PNGs
            delete_frames(frames_dir);
        } else {
            enc_ms = 0;
        }

        // Save checkpoint (next chunk index)
        sim.save_checkpoint(&canvas, chunk + 1, checkpoint_path);

        let pct = (chunk + 1) * 100 / n_chunks;
        let pop = sim.cells.len();
        println!("  chunk {}/{n_chunks} done ({pct}%)  pop={pop}  sim={sim_ms}ms enc={enc_ms}ms", chunk+1);
        // Write stats for segment-watcher.sh to include in Discord messages
        let _ = fs::write("state/last_stats.txt",
            format!("pop={pop}\ntarget=2560\nrange=[1920,3200]\nsim_ms={sim_ms}\nenc_ms={enc_ms}\n"));

// [recovery] edit target not found, appending:
        Sim { cells, order: (0..n).collect(), rng, g, softening, speed_cap, start_pop: target_pop,
              pop_band, rate_limit, tick_count: 0, prev_live: vec![false; W * H], wrap, steer, dampen,

// [recovery] edit target not found, appending:
        fn new(rng_seed: u64, g: f32, softening: f32, speed_cap: f32, pop_band: f32,
           rate_limit: usize, seed_density_inv: usize, target_pop: usize, wrap: bool, steer: bool, dampen: bool, init_vel: &str) -> Self {

// [recovery] edit target not found, appending:
        println!("\n[chunk {}/{n_chunks}] frames {}..{}", chunk+1, chunk_start_frame, chunk_end_frame);
        // Hot-reload palette at chunk boundary — drop a file to change mid-run.
        let palette = load_palette();
        if !headless { println!("  palette: {:?}", palette); }
        let mut chunk_audio: Vec<f32> = Vec::with_capacity(SAMPLES_PER_FRAME * this_chunk_frames * 2); // stereo interleaved

        // Render frames for this chunk — check signal each frame
        let sim_t0 = std::time::Instant::now();
        for local_frame in 0..this_chunk_frames {
            if !keep_running.load(Ordering::Relaxed) {
                // Discard partial chunk and stop immediately
                println!("[signal] Discarding partial chunk {}, cleaning up {} frames...",
                    chunk + 1, local_frame);
                delete_frames(frames_dir);
                break 'chunks;
            }
            let global_frame = chunk_start_frame + local_frame;
            if !headless {
                sim.paint_frame(&mut canvas, &palette);
                Sim::save_png(&canvas, &format!("{frames_dir}/f{global_frame:013}.png"));

// [recovery] edit target not found, appending:
            sim.paint_frame(&mut canvas, &palette);
            let global_frame = total_frames + ep_frame;
            let path = format!("{frames_dir}/f{global_frame:013}.png");

// [recovery] edit target not found, appending:
// ── Colour system ────────────────────────────────────────────────────────────
//
// "Radical" (default): perceptual hue-wheel interpolation.
//   • Velocity direction θ → position on a circular interpolation of the 9
//     palette colours (sorted by OKLCH hue; achromatic colours with C < 0.02
//     are excluded from the wheel).
//   • Speed ramp: #061B31 dark navy at t=0 → interpolated palette colour at
//     t=1 (speed_cap).  Linear in OKLab all the way through.
//   • t > 1 (beyond speed_cap, up to 2×): extrapolate — push L toward 0.92
//     and scale C outward, both with a √-taper for diminishing returns.
//   • Out-of-gamut handling: OKLCH binary-search chroma reduction (8
//     iterations) — hue and lightness are preserved, only C is squeezed.
//     This is the CSS Color Level 4 gamut-mapping algorithm.
//
// "Classic" (write "classic" to /tmp/gravity_palette): original uniform wheel.
//
// Hot-reload: binary reads /tmp/gravity_palette at each chunk boundary.
// Default (file absent or unrecognised): "radical".

/// One anchor on the hue wheel (OKLab + precomputed OKLCH hue).
#[derive(Clone)]
struct HueAnchor { l: f32, a: f32, b: f32, h: f32 }

/// The 9 palette colours (#061B31 also serves as the zero-speed anchor).
const PALETTE_SRGB: [(u8, u8, u8); 9] = [
    (0x53, 0x3A, 0xFD), // #533AFD — violet
    (0x06, 0x1B, 0x31), // #061B31 — dark navy  (also slow anchor)
    (0x50, 0x61, 0x7A), // #50617A — steel blue-gray
    (0xF6, 0xF9, 0xFC), // #F6F9FC — near white  (skipped: C < 0.02)
    (0xFF, 0xC0, 0x1F), // #FFC01F — golden yellow
    (0xFF, 0x61, 0x18), // #FF6118 — orange
    (0xF4, 0x4B, 0xCC), // #F44BCC — hot pink
    (0xEA, 0x22, 0x61), // #EA2261 — crimson
    (0x63, 0x5B, 0xFF), // #635BFF — periwinkle
];

/// Zero-speed (still) anchor colour — dark navy.
const SLOW_RGB: (u8, u8, u8) = (0x06, 0x1B, 0x31);

fn srgb_u8_to_linear(x: u8) -> f32 {
    let x = x as f32 / 255.0;
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

fn rgb_to_oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (rl, gl, bl) = (srgb_u8_to_linear(r), s

// [recovery] edit target not found, appending:
// ── Colour system ─────────────────────────────────────────────────────────────
//
// "Radical" mode (default): perceptual hue-wheel interpolation.
//   Velocity direction θ → position on a circular spline through the palette colours.
//   Speed ramp: #061B31 (dark navy, still) → palette colour at speed_cap.
//   Beyond speed_cap (cells can reach 2×): L and C extrapolated with √ taper.
//   Out-of-gamut colours → OKLCH chroma binary-search reduction (hue-preserving).
//
// "Classic" mode (write "classic" to /tmp/gravity_palette): original uniform hue wheel.
//
// Palette (9 colours). Near-achromatic ones (C < 0.02 in Oklch) are excluded from
// the hue wheel but #061B31 is used as the zero-speed anchor regardless.
//
//   #533AFD  violet           #635BFF  periwinkle
//   #F44BCC  hot pink         #EA2261  crimson
//   #FF6118  orange           #FFC01F  golden yellow
//   #50617A  steel blue-gray  #061B31  dark navy (zero-speed anchor)
//   #F6F9FC  near white       (excluded: C < 0.02)

const PALETTE_SRGB: &[(u8, u8, u8)] = &[
    (0x53, 0x3A, 0xFD), // #533AFD — violet
    (0x06, 0x1B, 0x31), // #061B31 — dark navy  (also zero-speed anchor)
    (0x50, 0x61, 0x7A), // #50617A — steel blue-gray
    (0xF6, 0xF9, 0xFC), // #F6F9FC — near white  (C < 0.02, skipped from wheel)
    (0xFF, 0xC0, 0x1F), // #FFC01F — golden yellow
    (0xFF, 0x61, 0x18), // #FF6118 — orange
    (0xF4, 0x4B, 0xCC), // #F44BCC — hot pink
    (0xEA, 0x22, 0x61), // #EA2261 — crimson
    (0x63, 0x5B, 0xFF), // #635BFF — periwinkle
];

/// Zero-speed (still cell) colour — dark navy.
const SLOW_RGB: (u8, u8, u8) = (0x06, 0x1B, 0x31);

fn srgb_u8_to_linear(x: u8) -> f32 {
    let x = x as f32 / 255.0;
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

fn rgb_to_oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (rl, gl, bl) = (srgb_u8_to_linear(r), srgb_u8_to_linear(g), srgb_u8_to_linear(b));
    let lms_l = 0.4122214708 * rl + 0.5363325363 * gl + 0.0514459929 * bl;
    let lms_m = 0.2119034982 * rl + 0.6806995451 * gl + 0.1073969566 * bl;
    let lms_s = 0.0883024619 * rl + 0.2817188376 * gl + 0.6299787005 * bl;
    let (l_, m_, s_) = (lms_l.cbrt(), lms_m.cbrt(), lms_s.cbrt());
    let lab_l =  0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let lab_a =  1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let lab_b =  0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
    (lab_l, lab_a, lab_b)
}

/// Directional colour anchors (OKLab).  Velocity direction selects four basis colours
/// via squared-clamp weights that sum to 1 on the unit circle:
///   w_right = max(rx, 0)²   w_left = max(−rx, 0)²
///   w_down  = max(ry, 0)²   w_up   = max(−ry, 0)²
/// where (rx, ry) is the velocity rotated by `wheel_rotation` turns
/// (negative = CCW in screen space).  Weights sum to 1 naturally; no normalisation needed.
///
/// right / left  → blue family (#635BFF periwinkle / #533AFD violet)
/// down  / up    → warm family (#F44BCC hot-pink    / #F6F9FC near-white)
/// diagonal blends give intermediate colours.
/// zero-speed anchor: #061B31 dark navy.
#[derive(Clone, Debug)]
struct DirectionalPalette {
    dark:           (f32, f32, f32),  // #061B31  dark navy — slow/still anchor
    c_right:        (f32, f32, f32),  // #635BFF  periwinkle — +x
    c_left:         (f32, f32, f32),  // #533AFD  violet      — −x
    c_down:         (f32, f32, f32),  // #061B31  dark navy  — +y (screen-down)
    c_up:           (f32, f32, f32),  // #F6F9FC  near-white — −y (screen-up)
    wheel_rotation:       f32,   // turns; negative = CCW in screen space
    pos_rotation_enabled: bool,  // apply position-based hue rotation to velocity input
    pos_rotation_output:  bool,  // also rotate output (a,b) by same angle (default: off)
}

impl DirectionalPalette {
    fn build(pos_rotation_enabled: bool, pos_rotation_output: bool) -> Self {
        DirectionalPalette {
            dark:                rgb_to_oklab(0x06, 0x1B, 0x31),  // dark navy — zero-speed anchor
            c_right:             rgb_to_oklab(0x63, 0x5B, 0xFF),  // periwinkle blue — +x
            c_left:              rgb_to_oklab(0x53, 0x3A, 0xFD),  // violet          — −x
            c_down:              rgb_to_oklab(0x06, 0x1B, 0x31),  // dark navy       — +y
            c_up:                rgb_to_oklab(0xF6, 0xF9, 0xFC),  // cool near-white — −y
            wheel_rotation:      -11.0 / 360.0,  // 11° CCW — current scheme
            pos_rotation_enabled,
            pos_rotation_output,
        }
    }

    /// Blend the four directional anchors for a unit velocity (ux, uy).
    /// Rotation = scheme wheel_rotation + optional position-based rotation:
    ///   max 1 turn total; axes weighted by W/(W+H) and H/(W+H) respectively.
    ///   Formula: px/W + (py/H)*3 — 1 turn across width, 3 turns across height.
    fn directional_color(&self, ux: f32, uy: f32, px: f32, py: f32) -> (f32, f32, f32) {
        let pos_rot = if self.pos_rotation_enabled {
            // 1 full turn across width, 3 full turns across height.
            px / W as f32 + (py / H as f32) * 3.0
        } else { 0.0 };
        let angle = (self.wheel_rotation + pos_rot) * 2.0 * std::f32::consts::PI;
        let (ca, sa) = (angle.cos(), angle.sin());
        // Screen-space CCW rotation: rx = ux·cos + uy·sin, ry = −ux·sin + uy·cos
        let rx =  ux * ca + uy * sa;
        let ry = -ux * sa + uy * ca;
        let w_r = rx.max(0.0).powi(2);
        let w_l = (-rx).max(0.0).powi(2);
        let w_d = ry.max(0.0).powi(2);
        let w_u = (-ry).max(0.0).powi(2);
        // w_r + w_l + w_d + w_u = rx²+ ry² = |rotated unit vector|² = 1 — no normalisation needed
        let l = w_r*self.c_right.0 + w_l*self.c_left.0 + w_d*self.c_down.0 + w_u*self.c_up.0;
        let a = w_r*self.c_right.1 + w_l*self.c_left.1 + w_d*self.c_down.1 + w_u*self.c_up.1;
        let b = w_r*self.c_right.2 + w_l*self.c_left.2 + w_d*self.c_down.2 + w_u*self.c_up.2;
        (l, a, b)
    }
}

#[derive(Clone, Debug)]
enum PaletteMode {
    /// Original: speed→L/C, direction→hue uniformly (fallback).
    Classic,
    /// 4-directional weights: left/right=blue, up/down=warm, zero-speed=dark navy.
    Radical(DirectionalPalette),
}

fn load_palette(pos_rotation_enabled: bool, pos_rotation_output: bool) -> PaletteMode {
    let raw = std::fs::read_to_string("/tmp/gravity_palette").unwrap_or_default();
    let s = raw.trim().to_lowercase();
    if s.starts_with("classic") {
        PaletteMode::Classic
    } else {
        PaletteMode::Radical(DirectionalPalette::build(pos_rotation_enabled, pos_rotation_output))
    }
}

fn velocity_color_oklab(vx: f32, vy: f32, speed_cap: f32, palette: &PaletteMode) -> (f32, f32, f32) {
    let spd = (vx * vx + vy * vy).sqrt();

    match palette {
        PaletteMode::Classic => {
            let t = (spd / (speed_cap * 0.5)).clamp(0.0, 1.0);
            let l = 0.45 + 0.30 * t;
            let c = 0.20 * t;
            let h = vy.atan2(vx);
            (l, c * h.cos(), c * h.sin())
        }

        PaletteMode::Radical(dp) => {
            // t = 0 → still (dark navy), t = 1 → speed_cap, up to ~2.0 beyond.
            let t = spd / speed_cap;

            // Position-based rotation (same formula as directional_color input rotation).
            // Applied twice: once to input (inside directional_color), once to output ab.
            let pos_rot = if dp.pos_rotation_enabled {
                px / W as f32 + (py / H as f32) * 3.0
            } else { 0.0 };

            // Directional blend: unit velocity selects among four palette colours.
            let (dl, da, db) = dp.dark;
            let (tgt_l, tgt_a, tgt_b) = if spd > 1e-6 {
                dp.directional_color(vx / spd, vy / spd, px, py)
            } else {
                (dl, da, db)
            };

            let (l, a, b) = if t <= 1.0 {
                // Linear blend in OKLab: dark navy → directional palette colour.
                let l = dl + (tgt_l - dl) * t;
                let a = da + (tgt_a - da) * t;
                let b = db + (tgt_b - db) * t;
                (l, a, b)
            } else {
                // Beyond speed_cap: push L brighter and C more saturated.
                let extra = (t - 1.0).clamp(0.0, 1.0).sqrt();
                let l = (tgt_l + (0.92 - tgt_l) * extra * 0.45).min(0.93);
                let c_scale = 1.0 + extra * 0.40;
                (l, tgt_a * c_scale, tgt_b * c_scale)
            };

            // Output hue rotation: only when explicitly enabled (--pos-color-out).
            // wheel_rotation is input-only by default; pos_rot also drives output when opted in.
            if dp.pos_rotation_output {
                let out_angle = (dp.wheel_rotation + pos_rot) * 2.0 * std::f32::consts::PI;
                let (oca, osa) = (out_angle.cos(), out_angle.sin());
                (l, a * oca - b * osa, a * osa + b * oca)
            } else {
                (l, a, b)
            }
        }
    }
}

// ── sRGB conversion with hue-preserving gamut compression ─────────────────────

/// Oklab → linear sRGB (values may be outside [0, 1] for out-of-gamut colours).
fn oklab_to_linear_rgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    let r_lin =  4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g_lin = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let b_lin = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;
    (r_lin, g_lin, b_lin)
}

fn linear_to_srgb_u8(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let g = if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
    (g * 255.0).round() as u8
}

fn oklab_to_srgb(l: f32, a: f32, b: f32) -> (u8, u8, u8) {
    let (r, g, b_) = oklab_to_linear_rgb(l, a, b);

    // Fast path: in-gamut (the common case).
    if r >= 0.0 && r <= 1.0 && g >= 0.0 && g <= 1.0 && b_ >= 0.0 && b_ <= 1.0 {
        return (linear_to_srgb_u8(r), linear_to_srgb_u8(g), linear_to_srgb_u8(b_));
    }

    // Out-of-gamut: reduce chroma via binary search in OKLCH while preserving hue and L.
    // 8 iterations → precision of C to within C/256, imperceptible.
    let c0 = (a * a + b * b).sqrt();
    let h  = b.atan2(a);
    let (mut c_lo, mut c_hi) = (0.0_f32, c0);
    for _ in 0..8 {
        let c_mid = (c_lo + c_hi) * 0.5;
        let (a_m, b_m) = (c_mid * h.cos(), c_mid * h.sin());
        let (r2, g2, b2) = oklab_to_linear_rgb(l, a_m, b_m);
        if r2 >= 0.0 && r2 <= 1.0 && g2 >= 0.0 && g2 <= 1.0 && b2 >= 0.0 && b2 <= 1.0 {
            c_lo = c_mid;
        } else {
            c_hi = c_mid;
        }
    }
    let (a_s, b_s) = (c_lo * h.cos(), c_lo * h.sin());
    let (rs, gs, bs) = oklab_to_linear_rgb(l, a_s, b_s);
    (linear_to_srgb_u8(rs), linear_to_srgb_u8(gs), linear_to_srgb_u8(bs))
}

// [recovery] edit target not found, appending:
            let idx = yi * W + xi;
            if !occupied[idx] {
                let (vx, vy) = make_vel(xi, yi, &mut rng);
                let (vx, vy) = (vx * vel_scale, vy * vel_scale);
                cells.push(Cell { px: xi as f32 + 0.5, py: yi as f32 + 0.5, vx, vy,
                                  prev_speed: 0.0, id: next_id, moved: false });
                next_id += 1;
                occupied[idx] = true;
                seeded += 1;
            }
        }
        } // end else (random scatter)
        shuffle_vec(&mut cells, &mut rng);

// [recovery] edit target not found, appending:
              conway_births: 0, conway_deaths: 0, next_id,
              bucket_stats: [BucketStats::default(); 8],
              dir_voices: std::array::from_fn(|i| DirVoice::new(BUCKET_FREQS[i])),
              reverb: Reverb::new() }

// [recovery] edit target not found, appending:
                        next_id,
                        region_stats: [RegionStats::default(); 9],
                        region_voices: std::array::from_fn(|i| RegionVoice::new(
                            REGION_FREQS[i], REGION_PAN[i % 3], REGION_REVERB[i / 3])),
                        reverb: Reverb::new() };

// [recovery] edit target not found, appending:
    // Recommended invocation: --wrap --dampen-y 1.0 (vertical COM drift removal).
    // Good first defaults (may need tuning):
    //   G=0.075  soft=3  cap=2  pop=768  band=256  rate=16  vel=swirl  wrap  --dampen-y 1.0

// [recovery] edit target not found, appending:
        // Axis gravity scale: (1 - dampen) fraction of normal force on each axis.
        // Clamped to [0,1] so dampen>=1 means no gravity on that axis (not reversed).
        let gx_scale = (1.0 - self.dampen_x).clamp(0.0, 1.0);
        let gy_scale = (1.0 - self.dampen_y).clamp(0.0, 1.0);
        for i in 0..n {
            let (px, py) = (self.cells[i].px, self.cells[i].py);
            let (gfx, gfy) = qt_force(&nodes, 0, i, px, py, self.g, self.softening, self.wrap_x, self.wrap_y);
            self.cells[i].vx += gfx * gx_scale;
            self.cells[i].vy += gfy * gy_scale;
        }

        for c in &mut self.cells {
            // Isotropic speed cap (unchanged)
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            let effective_cap = c.prev_speed.max(self.speed_cap);
            if spd > effective_cap {
                c.vx = c.vx / spd * effective_cap;
                c.vy = c.vy / spd * effective_cap;
            }
            let hard_ceil = self.speed_cap * 2.0;
            c.prev_speed = c.prev_speed.min(spd).max(self.speed_cap).min(hard_ceil);
            // Per-axis speed cap: dampen axis gets a proportionally lower ceiling
            let vx_cap = self.speed_cap * gx_scale;
            let vy_cap = self.speed_cap * gy_scale;
            c.vx = c.vx.clamp(-vx_cap, vx_cap);
            c.vy = c.vy.clamp(-vy_cap, vy_cap);
        }

// [recovery] edit target not found, appending:
            let (gfx, gfy) = qt_force(&nodes, 0, i, px, py, self.g * g_scale, self.softening, self.wrap_x, self.wrap_y);

// [recovery] edit target not found, appending:
              region_stats: [RegionStats::default(); 9],
              region_voices: std::array::from_fn(|i| RegionVoice::new(
                  REGION_FREQS[i], REGION_PAN[i % 3], REGION_REVERB[i / 3])),
              reverb: Reverb::new() }
    }

// [recovery] edit target not found, appending:
/// right / left  → blue vs orange (#635BFF periwinkle / #FF9B3B orange-gold — opposite hues)
/// down  / up    → pink family  (#F44BCC hot-pink    / #FAF0F5 soft blush)
/// zero-speed anchor: #061B31 dark navy.
