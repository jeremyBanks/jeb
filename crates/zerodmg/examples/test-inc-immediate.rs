//! Test INC_16(BC) immediately followed by memory store

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("test-inc-immediate.gb", &rom).expect("Failed to write ROM");
    println!("Generated test-inc-immediate.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'I', b'N', b'C', b'I', b'M', b'M', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
    let instructions = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        LD_16_IMMEDIATE(BC, 0),
        
        // First INC
        INC_16(BC),
        
        // Store BC immediately
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, C),
        LD_8_INTERNAL(AT_HL, A),
        
        // Second INC
        INC_16(BC),
        
        // Store BC immediately
        LD_16_IMMEDIATE(HL, 0xC101),
        LD_8_INTERNAL(A, C),
        LD_8_INTERNAL(AT_HL, A),
        
        // Do a subtraction (like in division loop)
        LD_8_IMMEDIATE(E, 210),
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        
        // Third INC
        INC_16(BC),
        
        // Store BC immediately
        LD_16_IMMEDIATE(HL, 0xC102),
        LD_8_INTERNAL(A, C),
        LD_8_INTERNAL(AT_HL, A),
        
        HALT,
        JR(-1),
    ];
    
    for inst in instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

use Instruction::*;
use U8Register::*;
use U16Register::*;
use FlagCondition::*;

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
