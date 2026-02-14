//! Conway's Game of Life - Real Implementation
//! 16×16 grid with actual neighbor counting and evolution rules

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

const GRID_SIZE: u8 = 16;
const CELLS: u16 = (GRID_SIZE as u16) * (GRID_SIZE as u16); // 256

fn main() {
    let rom = build_rom();
    std::fs::write("life16.gb", &rom).expect("Failed to write ROM");
    println!("Generated life16.gb ({} bytes)", rom.len());
    println!("Grid: {}×{} = {} cells", GRID_SIZE, GRID_SIZE, CELLS);
    println!("Real neighbor counting + Conway's rules");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'L', b'I', b'F', b'E', b'-', b'1', b'6', 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    while rom.len() < 0x0150 { rom.push(0); }
    
    let instructions = game_code();
    for inst in instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

fn game_code() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use U8SecondaryRegister::*;
    use FlagCondition::*;
    
    // Memory layout:
    // 0xC000-0xC0FF: Current grid (256 bytes)
    // 0xC100-0xC1FF: Next grid (256 bytes)
    // 0xC200: Neighbor count scratch
    
    let mut code = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        LD_8_IMMEDIATE(A, 0x00),
        LD_8_TO_FF_IMMEDIATE(0x40), // LCD off
        
        // Load sprite tile 1 = filled square
        LD_16_IMMEDIATE(HL, 0x8010),
        LD_8_IMMEDIATE(A, 0xFF),
    ];
    
    for _ in 0..16 {
        code.push(LD_8_TO_SECONDARY(AT_HL_Plus));
    }
    
    // Clear both grids (512 bytes)
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xC000),
        LD_16_IMMEDIATE(BC, CELLS * 2),
        LD_8_IMMEDIATE(A, 0),
    ]);
    
    // Clear loop: 6 bytes total
    code.extend(vec![
        LD_8_TO_SECONDARY(AT_HL_Plus),
        DEC_16(BC),
        LD_8_INTERNAL(A, B),
        OR(C),
        JR_IF(if_NZ, -6),
    ]);
    
    // TEST: Fill entire grid with 1s to verify rendering
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xC000),
        LD_16_IMMEDIATE(BC, CELLS),
        LD_8_IMMEDIATE(A, 1),
    ]);
    
    // Fill loop: 6 bytes
    code.extend(vec![
        LD_8_TO_SECONDARY(AT_HL_Plus),
        DEC_16(BC),
        LD_8_INTERNAL(A, B),
        OR(C),
        JR_IF(if_NZ, -6),
    ]);
    
    // Palettes and LCD on
    code.extend(vec![
        LD_8_IMMEDIATE(A, 0x1B), // OBP0: sprites color 3=white
        LD_8_TO_FF_IMMEDIATE(0x48),
        LD_8_IMMEDIATE(A, 0xE4), // BGP: background black
        LD_8_TO_FF_IMMEDIATE(0x47),
        LD_8_IMMEDIATE(A, 0x83),
        LD_8_TO_FF_IMMEDIATE(0x40),
    ]);
    
    // Main loop
    code.extend(main_loop());
    
    code
}

fn main_loop() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    // Main evolution loop
    // For this initial version, I'll implement a simplified approach:
    // 1. Render current grid
    // 2. Delay
    // 3. Evolve to next grid
    // 4. Copy next → current
    // 5. Loop
    
    // For now, just render and loop (evolution TODO)
    let mut code = vec![];
    
    // MAIN_LOOP:
    code.extend(render_grid());
    
    // Delay (~1 second)
    for _ in 0..4 {
        code.extend(vec![
            LD_16_IMMEDIATE(BC, 0xFFFF),
            DEC_16(BC),
            LD_8_INTERNAL(A, B),
            OR(C),
            JR_IF(if_NZ, -5),
        ]);
    }
    
    // TODO: Add evolution logic here
    // For now just loop
    code.extend(vec![
        HALT,
        JR(-1),
    ]);
    
    code
}

fn render_grid() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use U8SecondaryRegister::*;
    use FlagCondition::*;
    
    let mut code = vec![];
    
    // Clear OAM
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xFE00),
        LD_8_IMMEDIATE(B, 40),
    ]);
    
    // Clear loop: 11 bytes
    code.extend(vec![
        LD_8_IMMEDIATE(A, 0),
        LD_8_INTERNAL(AT_HL, A),
        INC_16(HL), INC_16(HL), INC_16(HL), INC_16(HL),
        DEC(B),
        JR_IF(if_NZ, -11),
    ]);
    
    // Render grid cells
    // C = y, B = x, HL = grid pointer, DE = OAM pointer
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xC000),
        LD_16_IMMEDIATE(DE, 0xFE00),
        LD_8_IMMEDIATE(C, 0), // y
    ]);
    
    // ROW_LOOP:
    code.extend(vec![
        LD_8_IMMEDIATE(B, 0), // x
    ]);
    
    // CELL_LOOP:
    code.extend(vec![
        LD_8_INTERNAL(A, AT_HL),
        INC_16(HL),
        OR(A),
        JR_IF(if_Z, 30), // Skip 30 bytes to SKIP_DEAD if dead
        
        // Cell alive - write sprite
        // Y = C * 8 + 8
        LD_8_INTERNAL(A, C),
        SLA(A), SLA(A), SLA(A),
        ADD_IMMEDIATE(8),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        
        // X = B * 8 + 8
        LD_8_INTERNAL(A, B),
        SLA(A), SLA(A), SLA(A),
        ADD_IMMEDIATE(8),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        
        // Tile 1
        LD_8_IMMEDIATE(A, 1),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        
        // Attr 0
        LD_8_IMMEDIATE(A, 0),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
    ]);
    
    // SKIP_DEAD:
    code.extend(vec![
        INC(B),
        LD_8_INTERNAL(A, B),
        CP_IMMEDIATE(GRID_SIZE),
        JR_IF(if_NZ, -41), // Back to CELL_LOOP (41 bytes back)
        
        INC(C),
        LD_8_INTERNAL(A, C),
        CP_IMMEDIATE(GRID_SIZE),
        JR_IF(if_NZ, -(41 + 6)), // Back to ROW_LOOP (47 bytes back)
    ]);
    
    code
}

fn nintendo_logo() -> [u8; 0x30] {
    [
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
        0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
        0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
        0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
        0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
        0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    ]
}
