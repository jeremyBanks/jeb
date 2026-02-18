//! Conway's Game of Life with REAL evolution
//! Grid: 20x18 = 360 cells, toroidal wrapping
//!
//! Memory layout:
//!   0xC000..0xC167 = current grid (360 bytes, 0=dead 1=alive)
//!   0xC200..0xC367 = next grid (360 bytes)
//!
//! Strategy:
//!   Evolution code is fully unrolled by Rust for all 360 cells.
//!   For each cell i:
//!     - Load 8 neighbor values using direct memory reads (precomputed with wrap)
//!     - Sum into A (neighbor count)
//!     - Apply Conway's rules
//!     - Write result to next grid
//!   Then copy next → current.
//!
//! Rendering: grid values → BG tile indices at 0x9800.
//!   Tile 0 = 0xFF bytes = white (dead), Tile 1 = 0x00 bytes = black (alive)
//!   Default BGP = 0xFC: color0=white, color3=black → tile 0 pixel 3 = white ✓
//!
//! Status: Rendering verified working. Evolution logic WIP.
//! Known issue: full evolve_unrolled() kills all cells — root cause TBD.
//! Current fallback: evolve_identity() (static glider, no evolution).
//!
//! Debug findings:
//! - gb_asm JP fix: JP targets need base_address=0x0150 added (was bug)
//! - BG rendering: confirmed working via conway-simple comparison
//! - Identity evolution: confirmed working (glider persists)
//! - Full unrolled evolution: logic verified correct on paper, but kills cells in emulator
//! - TODO: trace emulator execution with debug output to find discrepancy

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

const W: usize = 20;
const H: usize = 18;
const CELLS: usize = W * H; // 360

const GRID_CUR: u16 = 0xC000;
const GRID_NXT: u16 = 0xC200;

fn main() {
    let rom = build_rom();
    std::fs::write("conway-evolve.gb", &rom).expect("Failed");
    let code_section = &rom[0x0150..];
    let code_end = code_section.windows(8)
        .position(|w| w.iter().all(|&b| b == 0))
        .unwrap_or(code_section.len());
    println!("Generated conway-evolve.gb (~{} code bytes)", code_end);
    println!("Grid: {}×{} = {} cells, toroidal wrapping", W, H, CELLS);
    println!("Mode: identity evolution (static glider — real evolution WIP)");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    rom.extend_from_slice(&[
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
        0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
        0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
        0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
        0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
        0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    ]);
    rom.extend_from_slice(&[b'C', b'O', b'N', b'W', b'A', b'Y', b'!', 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    while rom.len() < 0x0150 { rom.push(0); }
    for inst in &game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    while rom.len() < 32768 { rom.push(0); }
    rom
}

/// Compute the toroidal neighbor address for cell (y, x) with offset (dy, dx)
fn neighbor_addr(y: usize, x: usize, dy: i32, dx: i32) -> u16 {
    let ny = ((y as i32 + dy).rem_euclid(H as i32)) as usize;
    let nx = ((x as i32 + dx).rem_euclid(W as i32)) as usize;
    GRID_CUR + (ny * W + nx) as u16
}

fn game_code() -> Vec<Instruction> {
    let mut asm = Assembler::new();

    // ===== INIT =====
    asm.inst(DI)
        .inst(LD_16_IMMEDIATE(SP, 0xFFFE));

    // LCD on early (emulator requires LCD on for VRAM writes to take effect)
    asm.inst(LD_8_IMMEDIATE(A, 0x91))
        .inst(LD_8_TO_FF_IMMEDIATE(0x40));

    // Tile 0 = 0xFF bytes (dead = white with default BGP=0xFC)
    // Tile 1 = 0x00 bytes (alive = black)
    asm.inst(LD_16_IMMEDIATE(HL, 0x8000))
        .inst(LD_8_IMMEDIATE(B, 16))
        .inst(LD_8_IMMEDIATE(A, 0xFF));
    asm.label("TILE0")
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC(B))
        .jr_cond(if_NZ, "TILE0");

    asm.inst(LD_8_IMMEDIATE(B, 16))
        .inst(LD_8_IMMEDIATE(A, 0x00));
    asm.label("TILE1")
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC(B))
        .jr_cond(if_NZ, "TILE1");

    // Clear grids
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR))
        .inst(LD_16_IMMEDIATE(BC, (CELLS * 2) as u16))
        .inst(LD_8_IMMEDIATE(A, 0));
    asm.label("CLR")
        .inst(LD_8_IMMEDIATE(A, 0))       // Reload A=0 each iteration (loop check clobbers A)
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "CLR");

    // Glider at rows 5-7, cols 5-7
    for (y, x) in [(5usize, 6usize), (6, 7), (7, 5), (7, 6), (7, 7)] {
        let addr = GRID_CUR + (y * W + x) as u16;
        asm.inst(LD_16_IMMEDIATE(HL, addr))
            .inst(LD_8_IMMEDIATE(A, 1))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }

    // ===== MAIN LOOP =====
    asm.label("MAIN");

    // --- RENDER: copy grid to BG tilemap ---
    // BG map at 0x9800 is 32 tiles wide; our grid is 20 wide → skip 12 per row
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR))
        .inst(LD_16_IMMEDIATE(DE, 0x9800))
        .inst(LD_8_IMMEDIATE(B, H as u8));
    asm.label("RROW")
        .inst(LD_8_IMMEDIATE(C, W as u8));
    asm.label("RCELL")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC(C))
        .jr_cond(if_NZ, "RCELL");
    // DE += 12 (stride skip to next BG map row)
    asm.inst(LD_8_INTERNAL(A, E))
        .inst(ADD_IMMEDIATE(12))
        .inst(LD_8_INTERNAL(E, A))
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(ADC(D))
        .inst(LD_8_INTERNAL(D, A))
        .inst(DEC(B))
        .jr_cond(if_NZ, "RROW");

    // --- EVOLVE: real Conway's Life ---
    evolve_unrolled(&mut asm);

    // --- COPY next → current ---
    asm.inst(LD_16_IMMEDIATE(HL, GRID_NXT))
        .inst(LD_16_IMMEDIATE(DE, GRID_CUR))
        .inst(LD_16_IMMEDIATE(BC, CELLS as u16));
    asm.label("COPY")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "COPY");

    // --- DELAY ---
    asm.inst(LD_16_IMMEDIATE(BC, 50000u16));
    asm.label("DELAY")
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "DELAY");

    asm.jp("MAIN");

    asm.assemble()
}

/// Identity evolution: copy current → next unchanged.
/// Produces a static (non-evolving) display of the initial state.
fn evolve_identity(asm: &mut Assembler) {
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR))
        .inst(LD_16_IMMEDIATE(DE, GRID_NXT))
        .inst(LD_16_IMMEDIATE(BC, CELLS as u16));
    asm.label("IDCPY")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "IDCPY");
}

/// Real Conway evolution — fully unrolled for all 360 cells.
///
/// For each cell i at (y, x):
///   Load 8 neighbor values (addresses precomputed with toroidal wrap)
///   Sum into A (neighbor count 0..8)
///   Apply rules: next = (count==3) || (alive && count==2)
///   Write to GRID_NXT + i
///
/// NOTE: Currently buggy — kills all cells. Logic verified correct on paper.
/// JP instruction targets use base_address=0x0150 (fixed in gb_asm.rs).
/// TODO: trace actual emulator execution to find discrepancy.
#[allow(dead_code)]
fn evolve_unrolled(asm: &mut Assembler) {
    let neighbor_deltas: [(i32, i32); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        ( 0, -1),           ( 0, 1),
        ( 1, -1), ( 1, 0), ( 1, 1),
    ];

    for i in 0..CELLS {
        let y = i / W;
        let x = i % W;

        let neighbors: Vec<u16> = neighbor_deltas.iter()
            .map(|&(dy, dx)| neighbor_addr(y, x, dy, dx))
            .collect();

        // Load neighbor count into A
        asm.inst(LD_8_FROM_MEMORY_IMMEDIATE(neighbors[0]));
        for &addr in &neighbors[1..] {
            asm.inst(LD_16_IMMEDIATE(HL, addr));
            asm.inst(ADD(AT_HL));
        }

        // Save count, load current state
        asm.inst(LD_8_INTERNAL(B, A));
        let cur_addr = GRID_CUR + i as u16;
        asm.inst(LD_16_IMMEDIATE(HL, cur_addr));
        asm.inst(LD_8_INTERNAL(C, AT_HL)); // C = current state (0 or 1)
        asm.inst(LD_8_INTERNAL(A, B));     // A = count

        // Apply Conway's rules
        let lbl_w1   = format!("EW1_{}", i);
        let lbl_done = format!("ED_{}", i);
        let lbl_end  = format!("EE_{}", i);

        asm.inst(CP_IMMEDIATE(3));
        asm.jp_cond(if_Z, &lbl_w1);   // count==3 → born/survive
        asm.inst(CP_IMMEDIATE(2));
        asm.jp_cond(if_NZ, &lbl_done); // count!=2,3 → die
        asm.inst(LD_8_INTERNAL(A, C));
        asm.inst(OR(A));
        asm.jp_cond(if_NZ, &lbl_w1);  // count==2, alive → survive

        asm.label(&lbl_done);
        let nxt_addr = GRID_NXT + i as u16;
        asm.inst(LD_16_IMMEDIATE(HL, nxt_addr));
        asm.inst(LD_8_IMMEDIATE(A, 0));
        asm.inst(LD_8_INTERNAL(AT_HL, A));
        asm.jp(&lbl_end);

        asm.label(&lbl_w1);
        asm.inst(LD_16_IMMEDIATE(HL, nxt_addr));
        asm.inst(LD_8_IMMEDIATE(A, 1));
        asm.inst(LD_8_INTERNAL(AT_HL, A));

        asm.label(&lbl_end);
    }
}
