// Z85 with execution tracing

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-trace.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-trace.gb - outputs markers");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'T', b'R', b'C', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
    let mut instructions = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Store 210
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
        
        // Output "1" - before division
        LD_8_IMMEDIATE(A, b'1'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
    ];
    
    // ONE division with detailed tracing
    instructions.extend(vec![
        // Init quotient
        LD_8_IMMEDIATE(A, 0),
        LD_16_IMMEDIATE(HL, 0xC110),
        LD_8_INTERNAL(AT_HL, A),
        
        // Output "2" - quotient initialized
        LD_8_IMMEDIATE(A, b'2'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        // Division loop
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 14),  // Jump to output section (14 bytes ahead)
        
        SUB_IMMEDIATE(85),
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(AT_HL, A),
        
        LD_16_IMMEDIATE(HL, 0xC110),
        LD_8_INTERNAL(A, AT_HL),
        INC(A),
        LD_8_INTERNAL(AT_HL, A),
        
        JR(-24),
        
        // Output "3" - exited loop
        LD_8_IMMEDIATE(A, b'3'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        // Get remainder
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        
        // Output "4" - got remainder
        PUSH_AF,  // Save remainder
        LD_8_IMMEDIATE(A, b'4'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        POP_AF,  // Restore remainder
        
        // Add '0'
        ADD_IMMEDIATE(b'0'),
        
        // Output "5" - added '0'
        PUSH_AF,
        LD_8_IMMEDIATE(A, b'5'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        POP_AF,
        
        // Output the actual character
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        // Newline
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        HALT,
        JR(-1),
    ]);
    
    for inst in instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
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
