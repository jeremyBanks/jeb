//! Trace INSIDE division loop to see if body executes

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("test-div-trace-inside.gb", &rom).expect("Failed to write ROM");
    println!("Generated test-div-trace-inside.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'D', b'I', b'V', b'I', b'N', b'S', b'I', b'D', b'E', 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
    let instructions = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        LD_16_IMMEDIATE(DE, 210),
        LD_16_IMMEDIATE(BC, 0),
        
        // Output '1' before loop
        LD_8_IMMEDIATE(A, b'1'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // LOOP_START
        LD_8_INTERNAL(A, D),
        OR(A),
        JR_IF(if_NZ, 5),
        
        // Output '2' (D==0 path)
        LD_8_IMMEDIATE(A, b'2'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        
        // Output '3' if C flag set (E < 85), '4' if not (E >= 85)
        LD_8_IMMEDIATE(A, b'3'),
        JR_IF(if_C, 2),
        LD_8_IMMEDIATE(A, b'4'),
        
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Now do the actual exit check
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 18), // Exit if E < 85 (adjusted for output above)
        
        // Output '5' (entering loop body)
        LD_8_IMMEDIATE(A, b'5'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Subtract
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 2),
        DEC(D),
        
        // Output '6' (before INC)
        LD_8_IMMEDIATE(A, b'6'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        INC_16(BC),
        
        // Output '7' (after INC)
        LD_8_IMMEDIATE(A, b'7'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Don't loop - just exit to see first iteration
        LD_8_INTERNAL(A, E),
        
        // Output final BC
        LD_8_INTERNAL(A, C),
        ADD_IMMEDIATE(48),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
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
