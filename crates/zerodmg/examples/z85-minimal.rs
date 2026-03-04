//! Minimal test: just output a byte to verify serial works

use zerodmg_codes::instruction::prelude::*;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-minimal.gb", &rom).expect("Failed");
    println!("Generated z85-minimal.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[b'M',b'I',b'N',0,0,0,0,0,0,0,0,0,0,0,0,0]);
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    while rom.len() < 0x0260 { rom.push(0); }
    for inst in vec![
        Instruction::DI,
        Instruction::LD_16_IMMEDIATE(U16Register::SP, 0xFFFE),
        Instruction::LD_8_IMMEDIATE(U8Register::A, 0x28), // Output 40
        Instruction::LD_8_TO_FF_IMMEDIATE(0x01),
        Instruction::PUSH_AF,
        Instruction::LD_8_IMMEDIATE(U8Register::A, 0x81),
        Instruction::LD_8_TO_FF_IMMEDIATE(0x02),
        Instruction::POP_AF,
        Instruction::HALT,
        Instruction::JR(-1),
    ] {
        rom.extend_from_slice(&inst.to_bytes());
    }
    while rom.len() < 32768 { rom.push(0); }
    rom
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
