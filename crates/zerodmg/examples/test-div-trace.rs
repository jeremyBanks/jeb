//! Trace through division step by step

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("test-div-trace.gb", &rom).expect("Failed to write ROM");
    println!("Generated test-div-trace.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'T', b'R', b'A', b'C', b'E', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
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
    use FlagCondition::*;
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Load 210 into DE
        LD_8_IMMEDIATE(E, 210),
        LD_8_IMMEDIATE(D, 0),
        
        // Initialize BC = 0
        LD_16_IMMEDIATE(BC, 0),
        
        // Output '1' - starting
        LD_8_IMMEDIATE(A, b'1'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Check: D > 0?
        LD_8_INTERNAL(A, D),
        OR(A),
        // Output '2' if D==0, '3' if D>0  
        LD_8_IMMEDIATE(A, b'2'),
        JR_IF(if_Z, 2),
        LD_8_IMMEDIATE(A, b'3'),
        
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Check: E >= 85?
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        // Output '4' if E >= 85, '5' if E < 85
        LD_8_IMMEDIATE(A, b'4'),
        JR_IF(if_NC, 2), 
        LD_8_IMMEDIATE(A, b'5'),
        
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Now do the subtraction: E = E - 85
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        
        // Output '6' if no borrow, '7' if borrow
        LD_8_IMMEDIATE(A, b'6'),
        JR_IF(if_NC, 2),
        LD_8_IMMEDIATE(A, b'7'),
        
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Increment BC
        INC_16(BC),
        
        // Output '8'
        LD_8_IMMEDIATE(A, b'8'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Store E at 0xC100
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, E),
        LD_8_INTERNAL(AT_HL, A),
        
        // Store C at 0xC101
        INC_16(HL),
        LD_8_INTERNAL(A, C),
        LD_8_INTERNAL(AT_HL, A),
        
        // Store B at 0xC102
        INC_16(HL),
        LD_8_INTERNAL(A, B),
        LD_8_INTERNAL(AT_HL, A),
        
        // Newline
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        HALT,
        JR(-1),
    ]
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
