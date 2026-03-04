//! Simple serial output test - just outputs "HELLO\n"

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

fn main() {
    let rom = build_rom();
    std::fs::write("serial-hello.gb", &rom).expect("Failed to write ROM");
    println!("Generated serial-hello.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'H', b'E', b'L', b'L', b'O', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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
    
    // Output "HELLO\n"
    let message = b"HELLO\n";
    for &byte in message {
        asm.inst(LD_8_IMMEDIATE(A, byte))
            .inst(LD_8_TO_FF_IMMEDIATE(0x01))
            .inst(PUSH_AF)
            .inst(LD_8_IMMEDIATE(A, 0x81))
            .inst(LD_8_TO_FF_IMMEDIATE(0x02))
            .inst(POP_AF);
    }
    
    asm.label("HALT")
        .inst(HALT)
        .jr("HALT");
    
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
