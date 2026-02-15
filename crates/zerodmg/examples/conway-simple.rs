//! Simple Conway's Life - static glider pattern (no evolution yet)
//! Just to verify rendering works

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

const GRID_WIDTH: usize = 20;
const GRID_HEIGHT: usize = 18;

fn main() {
    let rom = build_rom();
    std::fs::write("conway-simple.gb", &rom).expect("Failed to write ROM");
    println!("Generated conway-simple.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'C', b'O', b'N', b'W', b'A', b'Y', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0150 { rom.push(0); }
    
    let instructions = game_code();
    for inst in &instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

fn game_code() -> Vec<Instruction> {
    let mut asm = Assembler::new();
    
    asm.inst(DI)
        .inst(LD_16_IMMEDIATE(SP, 0xFFFE));
    
    // Initialize video
    asm.inst(LD_8_IMMEDIATE(A, 0x91))  // LCD on, BG on
        .inst(LD_8_TO_FF_IMMEDIATE(0x40));
    
    // Clear grid
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(LD_16_IMMEDIATE(BC, 360));
    asm.label("CLEAR")
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(INC_16(HL))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "CLEAR");
    
    // Glider pattern at (5,5)
    asm.inst(LD_16_IMMEDIATE(HL, (0xC000 + 5 * GRID_WIDTH + 6) as u16))
        .inst(LD_8_IMMEDIATE(A, 1))
        .inst(LD_8_INTERNAL(AT_HL, A));
    asm.inst(LD_16_IMMEDIATE(HL, (0xC000 + 6 * GRID_WIDTH + 7) as u16))
        .inst(LD_8_IMMEDIATE(A, 1))
        .inst(LD_8_INTERNAL(AT_HL, A));
    asm.inst(LD_16_IMMEDIATE(HL, (0xC000 + 7 * GRID_WIDTH + 5) as u16))
        .inst(LD_8_IMMEDIATE(A, 1))
        .inst(LD_8_INTERNAL(AT_HL, A));
    asm.inst(LD_16_IMMEDIATE(HL, (0xC000 + 7 * GRID_WIDTH + 6) as u16))
        .inst(LD_8_IMMEDIATE(A, 1))
        .inst(LD_8_INTERNAL(AT_HL, A));
    asm.inst(LD_16_IMMEDIATE(HL, (0xC000 + 7 * GRID_WIDTH + 7) as u16))
        .inst(LD_8_IMMEDIATE(A, 1))
        .inst(LD_8_INTERNAL(AT_HL, A));
    
    // Render loop
    asm.label("LOOP");
    
    // Render grid to tilemap
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(LD_16_IMMEDIATE(DE, 0x9800))
        .inst(LD_8_IMMEDIATE(B, GRID_HEIGHT as u8));
    
    asm.label("ROW")
        .inst(LD_8_IMMEDIATE(C, GRID_WIDTH as u8));
    
    asm.label("CELL")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))  // Tile 0=dead, 1=alive
        .inst(INC_16(DE))
        .inst(DEC(C))
        .jr_cond(if_NZ, "CELL");
    
    // Next row (skip 12 bytes to get to next screen row)
    asm.inst(LD_8_INTERNAL(A, E))
        .inst(ADD_IMMEDIATE(12))
        .inst(LD_8_INTERNAL(E, A))
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(ADC(D))
        .inst(LD_8_INTERNAL(D, A))
        .inst(DEC(B))
        .jr_cond(if_NZ, "ROW");
    
    asm.jr("LOOP");
    
    asm.assemble()
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
