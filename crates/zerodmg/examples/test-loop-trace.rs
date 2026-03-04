//! Trace if loop runs

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("test-loop-trace.gb", &rom).expect("Failed to write ROM");
    println!("Generated test-loop-trace.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'L', b'O', b'O', b'P', b'T', b'R', b'C', 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
    let instructions = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // DE = 210
        LD_16_IMMEDIATE(DE, 210),
        LD_16_IMMEDIATE(BC, 0),
        
        // Output '1' before loop
        LD_8_IMMEDIATE(A, b'1'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // === LOOP START ===
        LD_8_INTERNAL(A, D),
        OR(A),
        JR_IF(if_NZ, 5),
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 27), // Adjusted for trace output (+17 bytes)
        
        // Output '2' (loop body entered)
        PUSH(BC),
        PUSH(DE),
        LD_8_IMMEDIATE(A, b'2'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP(DE),
        POP(BC),
        
        // Subtract
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 2),
        DEC(D),
        
        // Output '3' (before INC)
        PUSH(BC),
        PUSH(DE),
        LD_8_IMMEDIATE(A, b'3'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP(DE),
        POP(BC),
        
        INC_16(BC),
        
        // Output BC after first INC
        PUSH(BC),
        PUSH(DE),
        LD_8_INTERNAL(A, C),
        ADD_IMMEDIATE(48),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP(DE),
        POP(BC),
        
        // Check again: E >= 85?
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 27), // Exit if E < 85
        
        // Second iteration
        PUSH(BC),
        PUSH(DE),
        LD_8_IMMEDIATE(A, b'5'), // Second loop marker
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP(DE),
        POP(BC),
        
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 2),
        DEC(D),
        INC_16(BC),
        
        // Output BC after second INC
        PUSH(BC),
        PUSH(DE),
        LD_8_IMMEDIATE(A, b'6'), // Marker
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP(DE),
        POP(BC),
        
        PUSH(BC),
        PUSH(DE),
        LD_8_INTERNAL(A, C),
        ADD_IMMEDIATE(48),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP(DE),
        POP(BC),
        
        LD_8_INTERNAL(A, E),
        
        // Output '4' (after loop)
        LD_8_IMMEDIATE(A, b'4'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Output BC
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
