//! Conway's Game of Life - Two Orbiting Clumps with Fade Trails
//!
//! Two lightweight spaceships (LWSS) moving in opposite horizontal directions
//! on a 20×18 toroidal grid. History fades at 12.5% per tick (brightness >>= 3/8).
//!
//! Memory layout:
//!   0xC000..0xC167 = current grid (360 bytes, 0=dead 1=alive)
//!   0xC200..0xC367 = next grid (360 bytes)
//!   0xC400..0xC567 = fade values (360 bytes, 0..255, 255=bright, 0=black)
//!
//! Tiles (4 shades):
//!   Tile 0: all 0x00 bytes  → palette color 0 = black   (dead, cold)
//!   Tile 1: all 0x55/0xAA   → palette color 1 = dark     (fading)
//!   Tile 2: all 0xAA/0x55   → palette color 2 = light    (fading)
//!   Tile 3: all 0xFF bytes  → palette color 3 = white    (alive, hot)
//!   BGP = 0xE4: 11 10 01 00 → color3=black, color2=dark, color1=light, color0=white
//!   (inverted so "alive" = white on black background, fading to black)

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

const W: usize = 20;
const H: usize = 18;
const CELLS: usize = W * H; // 360

const GRID_CUR: u16 = 0xC000;
const GRID_NXT: u16 = 0xC200;
const GRID_FADE: u16 = 0xC400;

fn main() {
    let rom = build_rom();
    std::fs::write("conway-orbit.gb", &rom).expect("Failed to write ROM");
    let code_section = &rom[0x0150..];
    let code_end = code_section.windows(8)
        .position(|w| w.iter().all(|&b| b == 0))
        .unwrap_or(code_section.len());
    println!("Generated conway-orbit.gb ({} code bytes)", code_end);
    println!("Two LWSSes orbiting on {}×{} toroidal grid", W, H);
    println!("Fade trails at 12.5% per tick → 4 brightness levels");
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
    rom.extend_from_slice(&[b'O', b'R', b'B', b'I', b'T', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    while rom.len() < 0x0150 { rom.push(0); }
    for inst in &game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    while rom.len() < 32768 { rom.push(0); }
    rom
}

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

    // LCD on for VRAM access
    asm.inst(LD_8_IMMEDIATE(A, 0x91))
        .inst(LD_8_TO_FF_IMMEDIATE(0x40));

    // Set BGP palette: 0xE4 = color3=darkest, color2=dark, color1=light, color0=white
    // So tile index 0=black, 1=dark gray, 2=light gray, 3=white
    asm.inst(LD_8_IMMEDIATE(A, 0xE4))
        .inst(LD_8_TO_FF_IMMEDIATE(0x47)); // BGP

    // Load 4 BG tiles
    // Tile 0 (index 0): 0x00 all → color 0 → black (dead, fully faded)
    // Tile 1 (index 1): 0x55,0xAA pattern → color 1 → dark gray
    // Tile 2 (index 2): 0xAA,0x55 pattern → color 2 → light gray
    // Tile 3 (index 3): 0xFF all → color 3 → white (alive)
    // Each tile: 8 rows × 2 bytes = 16 bytes; each row = (low byte, high byte)
    // For solid color N: low=N_bits, high=N_bits where color=bit(high)<<1|bit(low)
    // color 0 = 00: low=0x00, high=0x00
    // color 1 = 01: low=0xFF, high=0x00
    // color 2 = 10: low=0x00, high=0xFF
    // color 3 = 11: low=0xFF, high=0xFF

    // Tile 0: all color 0 (black)
    asm.inst(LD_16_IMMEDIATE(HL, 0x8000))
        .inst(LD_8_IMMEDIATE(B, 16))
        .inst(LD_8_IMMEDIATE(A, 0x00));
    asm.label("T0L")
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC(B))
        .jr_cond(if_NZ, "T0L");

    // Tile 1: all color 1 (dark gray): low=0xFF, high=0x00 × 8 rows
    asm.inst(LD_8_IMMEDIATE(B, 8)); // 8 rows
    asm.label("T1L")
        .inst(LD_8_IMMEDIATE(A, 0xFF))
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus)) // low byte
        .inst(LD_8_IMMEDIATE(A, 0x00))
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus)) // high byte
        .inst(DEC(B))
        .jr_cond(if_NZ, "T1L");

    // Tile 2: all color 2 (light gray): low=0x00, high=0xFF × 8 rows
    asm.inst(LD_8_IMMEDIATE(B, 8));
    asm.label("T2L")
        .inst(LD_8_IMMEDIATE(A, 0x00))
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus)) // low byte
        .inst(LD_8_IMMEDIATE(A, 0xFF))
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus)) // high byte
        .inst(DEC(B))
        .jr_cond(if_NZ, "T2L");

    // Tile 3: all color 3 (white): low=0xFF, high=0xFF × 8 rows
    asm.inst(LD_8_IMMEDIATE(B, 8));
    asm.label("T3L")
        .inst(LD_8_IMMEDIATE(A, 0xFF))
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus)) // low
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus)) // high
        .inst(DEC(B))
        .jr_cond(if_NZ, "T3L");

    // Clear all three grid regions
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR))
        .inst(LD_16_IMMEDIATE(BC, (CELLS * 3) as u16));
    asm.label("CLR")
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "CLR");

    // ===== INITIAL STATE =====
    // Two LWSSes (lightweight spaceships) moving in opposite directions.
    //
    // LWSS shape (moving right):
    //   .OO..
    //   OOOO.
    //   OO.OO  ← anchor row
    //   ..OO.
    //
    // LWSS 1: top-center area, moving right (standard orientation)
    // Row 3-6, cols 4-8
    //
    // LWSS 2: bottom-center area, moving left (mirror horizontally)
    // Row 11-14, cols 11-15

    // LWSS 1 - moving right (NE quadrant)
    // Standard LWSS:
    //   row y+0: x+1, x+4
    //   row y+1: x+0
    //   row y+2: x+0, x+4
    //   row y+3: x+0, x+1, x+2, x+3
    let lwss1_base = (3usize, 3usize); // (row, col) top-left
    let lwss1_cells: &[(i32,i32)] = &[
        (0, 1), (0, 4),
        (1, 0),
        (2, 0), (2, 4),
        (3, 0), (3, 1), (3, 2), (3, 3),
    ];

    // LWSS 2 - moving left (SW quadrant), mirrored horizontally
    // Mirror: flip x within width 5: x' = 4-x
    let lwss2_base = (11usize, 12usize);
    let lwss2_cells: &[(i32,i32)] = &[
        (0, 3), (0, 0),   // mirrored (0,1)→(0,3), (0,4)→(0,0)
        (1, 4),           // mirrored (1,0)→(1,4)
        (2, 4), (2, 0),   // mirrored (2,0)→(2,4), (2,4)→(2,0)
        (3, 4), (3, 3), (3, 2), (3, 1), // mirrored row
    ];

    for &(dy, dx) in lwss1_cells {
        let y = lwss1_base.0 as i32 + dy;
        let x = lwss1_base.1 as i32 + dx;
        let addr = GRID_CUR + (y as usize * W + x as usize) as u16;
        asm.inst(LD_16_IMMEDIATE(HL, addr))
            .inst(LD_8_IMMEDIATE(A, 1))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }

    for &(dy, dx) in lwss2_cells {
        let y = lwss2_base.0 as i32 + dy;
        let x = lwss2_base.1 as i32 + dx;
        let addr = GRID_CUR + (y as usize * W + x as usize) as u16;
        asm.inst(LD_16_IMMEDIATE(HL, addr))
            .inst(LD_8_IMMEDIATE(A, 1))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }

    // Also initialize fade to 255 for alive cells
    for &(dy, dx) in lwss1_cells {
        let y = lwss1_base.0 as i32 + dy;
        let x = lwss1_base.1 as i32 + dx;
        let addr = GRID_FADE + (y as usize * W + x as usize) as u16;
        asm.inst(LD_16_IMMEDIATE(HL, addr))
            .inst(LD_8_IMMEDIATE(A, 255))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }
    for &(dy, dx) in lwss2_cells {
        let y = lwss2_base.0 as i32 + dy;
        let x = lwss2_base.1 as i32 + dx;
        let addr = GRID_FADE + (y as usize * W + x as usize) as u16;
        asm.inst(LD_16_IMMEDIATE(HL, addr))
            .inst(LD_8_IMMEDIATE(A, 255))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }

    // ===== MAIN LOOP =====
    asm.label("MAIN");

    // --- STEP 1: EVOLVE ---
    evolve_unrolled(&mut asm);

    // --- STEP 2: UPDATE FADE + COPY next→current ---
    // For each cell i:
    //   if next[i] == 1: fade[i] = 255
    //   else: fade[i] = fade[i] - (fade[i] >> 3)   [subtract 12.5%]
    // Then copy next→current
    //
    // We do this with a loop over all 360 cells
    // HL = GRID_NXT ptr, DE = GRID_CUR ptr, BC used as counter
    // We also need GRID_FADE ptr — store in a memory temp
    // Use 0xCF00 as a temp for fade pointer
    
    // Store fade pointer in memory (0xCF00 = low byte of GRID_FADE ptr = 0xC400+i)
    // We'll use a loop: i in 0..360
    //   HL = GRID_NXT + i (source)
    //   DE = GRID_CUR + i (dest)
    //   IX-equivalent: use stack? No IX on GB. Use a second HL via push/pop.
    // Strategy: use HL for GRID_NXT, shadow vars in fixed memory
    //   0xCF00 = current fade cell pointer (low), 0xCF01 = high
    //   Or simpler: compute fade addr from cell index using a fixed offset
    //   GRID_FADE - GRID_NXT = 0xC400 - 0xC200 = 0x200
    //   GRID_CUR  - GRID_NXT = 0xC000 - 0xC200 = -0x200 (underflow, needs signed)
    //   Simpler: just do 3 separate passes or use C for count, keep HL/DE/BC

    // We'll track:
    //   H:L = GRID_NXT pointer (reads next state)
    //   D:E = GRID_CUR pointer (writes current state)
    //   BC = cell counter (360 down to 0)
    // For fade: fade_addr = GRID_FADE + (cell_index)
    //           = GRID_FADE + (CELLS - BC) when BC counts down
    // But that requires computing the index... Let's use a fixed offset trick:
    //   GRID_FADE base - GRID_NXT base = 0xC400 - 0xC200 = 0x0200
    // So fade_addr = HL + 0x200 (but HL changes each iter)
    // Save HL to scratch, add 0x200, access, restore. Too many ops.
    //
    // Alternative: do the fade update as a separate loop AFTER the copy.
    // First loop: copy GRID_NXT → GRID_CUR (360 bytes)
    // Second loop: for each cell: read GRID_CUR (current=new state), update GRID_FADE

    // Loop 1: copy GRID_NXT → GRID_CUR
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

    // Loop 2: update fade values
    // HL = GRID_CUR (alive state), DE = GRID_FADE (fade values)
    // For each cell: if alive → fade=255 else fade -= fade>>3
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR))
        .inst(LD_16_IMMEDIATE(DE, GRID_FADE))
        .inst(LD_16_IMMEDIATE(BC, CELLS as u16));
    asm.label("FADE")
        .inst(LD_8_INTERNAL(A, AT_HL))  // A = alive state
        .inst(INC_16(HL))
        .inst(OR(A))
        .jr_cond(if_NZ, "FADESET");     // alive → set to 255
    // Dead: fade[i] = fade[i] - (fade[i] >> 3)
    .inst(LD_8_FROM_SECONDARY(AT_DE))   // A = fade[i]
    .inst(LD_8_INTERNAL(B, A))          // B = fade[i]
    .inst(SRA(A))                        // A >>= 1 (signed shift, but fade is 0-255, fine)
    .inst(SRA(A))                        // >>= 2
    .inst(SRA(A))                        // >>= 3 → A = fade[i] >> 3
    .inst(LD_8_INTERNAL(C, A))          // C = fade[i] >> 3
    .inst(LD_8_INTERNAL(A, B))          // A = fade[i]
    .inst(SUB(C))                        // A = fade[i] - (fade[i] >> 3)
    .jr_cond(if_C, "FADEZ");            // if underflow → 0
    .inst(LD_8_TO_SECONDARY(AT_DE))     // fade[i] = A
    .jp("FADENXT");
    asm.label("FADEZ")
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .jp("FADENXT");
    asm.label("FADESET")
        .inst(LD_8_IMMEDIATE(A, 255))
        .inst(LD_8_TO_SECONDARY(AT_DE));
    asm.label("FADENXT")
        .inst(INC_16(DE))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "FADE");

    // --- STEP 3: RENDER ---
    // Map fade[i] → tile index: fade >> 6 → 0..3
    // Tile 0=black(dead), 1=dark, 2=light, 3=white(alive)
    // Write to BG tilemap at 0x9800, stride 32
    asm.inst(LD_16_IMMEDIATE(HL, GRID_FADE))
        .inst(LD_16_IMMEDIATE(DE, 0x9800))
        .inst(LD_8_IMMEDIATE(B, H as u8));
    asm.label("RROW")
        .inst(LD_8_IMMEDIATE(C, W as u8));
    asm.label("RCELL")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(SRL(A))   // A >>= 1
        .inst(SRL(A))   // A >>= 2
        .inst(SRL(A))   // A >>= 3
        .inst(SRL(A))   // A >>= 4
        .inst(SRL(A))   // A >>= 5
        .inst(SRL(A))   // A >>= 6 → tile index 0..3
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC(C))
        .jr_cond(if_NZ, "RCELL");
    // stride skip: DE += 12 (32 - 20 = 12 to skip to next BG row)
    asm.inst(LD_8_INTERNAL(A, E))
        .inst(ADD_IMMEDIATE(12))
        .inst(LD_8_INTERNAL(E, A))
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(ADC(D))
        .inst(LD_8_INTERNAL(D, A))
        .inst(DEC(B))
        .jr_cond(if_NZ, "RROW");

    // --- STEP 4: DELAY ---
    asm.inst(LD_16_IMMEDIATE(BC, 30000u16));
    asm.label("DELAY")
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "DELAY");

    asm.jp("MAIN");

    asm.assemble()
}

/// Real Conway evolution — fully unrolled for all 360 cells.
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

        // Sum neighbors into A
        asm.inst(LD_8_FROM_MEMORY_IMMEDIATE(neighbors[0]));
        for &addr in &neighbors[1..] {
            asm.inst(LD_16_IMMEDIATE(HL, addr));
            asm.inst(ADD(AT_HL));
        }

        // B = count, C = current state
        asm.inst(LD_8_INTERNAL(B, A));
        let cur_addr = GRID_CUR + i as u16;
        asm.inst(LD_16_IMMEDIATE(HL, cur_addr));
        asm.inst(LD_8_INTERNAL(C, AT_HL));
        asm.inst(LD_8_INTERNAL(A, B));

        let lbl_w1   = format!("EW1_{}", i);
        let lbl_done = format!("ED_{}", i);
        let lbl_end  = format!("EE_{}", i);

        asm.inst(CP_IMMEDIATE(3));
        asm.jp_cond(if_Z, &lbl_w1);
        asm.inst(CP_IMMEDIATE(2));
        asm.jp_cond(if_NZ, &lbl_done);
        asm.inst(LD_8_INTERNAL(A, C));
        asm.inst(OR(A));
        asm.jp_cond(if_NZ, &lbl_w1);

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
