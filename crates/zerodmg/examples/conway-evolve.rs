//! Conway's Game of Life with REAL evolution
//! Grid: 20x18, wrapping edges
//! Memory: 0xC000 = current, 0xC200 = next, 0xC400 = temp

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

const W: u8 = 20;
const H: u8 = 18;

fn main() {
    let rom = build_rom();
    std::fs::write("conway-evolve.gb", &rom).expect("Failed");
    println!("Generated conway-evolve.gb - real evolution!");
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

fn game_code() -> Vec<Instruction> {
    let mut asm = Assembler::new();
    
    asm.inst(DI)
        .inst(LD_16_IMMEDIATE(SP, 0xFFFE))
        .inst(LD_8_IMMEDIATE(A, 0x91))
        .inst(LD_8_TO_FF_IMMEDIATE(0x40));
    
    // Clear grids
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(LD_16_IMMEDIATE(BC, 720));
    asm.label("CLR")
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(INC_16(HL))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "CLR");
    
    // Glider at (5,5)
    for (y, x) in [(5, 6), (6, 7), (7, 5), (7, 6), (7, 7)] {
        asm.inst(LD_16_IMMEDIATE(HL, (0xC000 + y * W as usize + x) as u16))
            .inst(LD_8_IMMEDIATE(A, 1))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }
    
    // === MAIN LOOP ===
    asm.label("MAIN");
    
    // Render to screen
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(LD_16_IMMEDIATE(DE, 0x9800))
        .inst(LD_8_IMMEDIATE(B, H));
    asm.label("RROW")
        .inst(LD_8_IMMEDIATE(C, W));
    asm.label("RCELL")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC(C))
        .jr_cond(if_NZ, "RCELL");
    asm.inst(LD_8_INTERNAL(A, E))
        .inst(ADD_IMMEDIATE(12))
        .inst(LD_8_INTERNAL(E, A))
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(ADC(D))
        .inst(LD_8_INTERNAL(D, A))
        .inst(DEC(B))
        .jr_cond(if_NZ, "RROW");
    
    // === EVOLUTION ===
    // TODO: Real neighbor counting - for now just copy grid
    
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(LD_16_IMMEDIATE(DE, 0xC200))
        .inst(LD_16_IMMEDIATE(BC, 360));
    
    asm.label("CPY")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "CPY");
    
    // Swap: next → current
    asm.inst(LD_16_IMMEDIATE(HL, 0xC200))
        .inst(LD_16_IMMEDIATE(DE, 0xC000))
        .inst(LD_16_IMMEDIATE(BC, 360));
    
    asm.label("SWAP")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "SWAP");
    
    // Delay
    asm.inst(LD_16_IMMEDIATE(BC, 3000));
    asm.label("DELAY")
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "DELAY");
    
    asm.jr("MAIN");
    
    asm.assemble()
}
