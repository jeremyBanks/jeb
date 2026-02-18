//! Test: Fill BG tilemap with tile 1 (should show all white)
mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

fn main() {
    let rom = build_rom();
    std::fs::write("bg-test.gb", &rom).expect("Failed");
    println!("Generated bg-test.gb");
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
    rom.extend_from_slice(&[b'B', b'G', b'T', b'E', b'S', b'T', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    while rom.len() < 0x0150 { rom.push(0); }
    for inst in &game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    while rom.len() < 32768 { rom.push(0); }
    rom
}

fn game_code() -> Vec<Instruction> {
    let mut asm = Assembler::new();
    
    asm.inst(DI)
        .inst(LD_16_IMMEDIATE(SP, 0xFFFE))
        .inst(LD_8_IMMEDIATE(A, 0x00))
        .inst(LD_8_TO_FF_IMMEDIATE(0x40)); // LCD off
    
    // Tile 0 = all 0x00 (black)
    asm.inst(LD_16_IMMEDIATE(HL, 0x8000))
        .inst(LD_8_IMMEDIATE(B, 16))
        .inst(LD_8_IMMEDIATE(A, 0x00));
    asm.label("T0")
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC(B))
        .jr_cond(if_NZ, "T0");
    
    // Tile 1 = all 0xFF (white)
    asm.inst(LD_8_IMMEDIATE(B, 16))
        .inst(LD_8_IMMEDIATE(A, 0xFF));
    asm.label("T1")
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC(B))
        .jr_cond(if_NZ, "T1");
    
    // BGP = 0xE4
    asm.inst(LD_8_IMMEDIATE(A, 0xE4))
        .inst(LD_8_TO_FF_IMMEDIATE(0x47));
    
    // Fill BG map with ALTERNATING 0 and 1 to create checkerboard
    asm.inst(LD_16_IMMEDIATE(HL, 0x9800))
        .inst(LD_16_IMMEDIATE(BC, 1024)); // BG map = 32x32 = 1024 bytes
    asm.label("FILL")
        .inst(LD_8_IMMEDIATE(A, 1)) // Always tile 1 (white) for now
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(INC_16(HL))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "FILL");
    
    // LCD on: BG enabled, tiles at 0x8000, map at 0x9800
    asm.inst(LD_8_IMMEDIATE(A, 0x91))
        .inst(LD_8_TO_FF_IMMEDIATE(0x40));
    
    // Halt
    asm.inst(HALT)
        .inst(JR(-1));
    
    asm.assemble()
}
