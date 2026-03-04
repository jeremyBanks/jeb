//! Minimal division test - just check final BC value in memory

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("test-div-minimal.gb", &rom).expect("Failed to write ROM");
    println!("Generated test-div-minimal.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'D', b'I', b'V', b'M', b'I', b'N', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
    let instructions = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Test value 210
        LD_16_IMMEDIATE(DE, 210),
        
        // Division (210 / 85 should give quotient=2, remainder=40)
        LD_16_IMMEDIATE(BC, 0),
        LD_8_INTERNAL(A, D),
        OR(A),
        JR_IF(if_NZ, 5),
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 10),
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 1), // FIX: Skip only DEC D, not INC BC!
        DEC(D),
        INC_16(BC),
        JR(-19),
        LD_8_INTERNAL(A, E),
        
        // Store BC to memory
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, C),
        LD_8_INTERNAL(AT_HL, A),
        INC_16(HL),
        LD_8_INTERNAL(A, B),
        LD_8_INTERNAL(AT_HL, A),
        
        // Store A (remainder) to memory
        LD_8_INTERNAL(A, E),
        LD_16_IMMEDIATE(HL, 0xC102),
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
