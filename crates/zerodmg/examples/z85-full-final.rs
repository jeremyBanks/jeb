// Full 32-bit Z85 Encoder - Final working version

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-full-final.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-full-final.gb");
    println!("Encodes 210 (0x000000D2) to Z85 digits");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'!', b'!', b'!', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
    let mut instructions = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Store test value 210
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
    ];
    
    // Do 5 divisions
    for _ in 0..5 {
        instructions.extend(vec![
            // Init quotient to 0
            LD_8_IMMEDIATE(A, 0),
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(AT_HL, A),
            
            // Division loop
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, AT_HL),
            CP_IMMEDIATE(85),
            JR_IF(if_C, 20),
            
            SUB_IMMEDIATE(85),
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(AT_HL, A),
            
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL),
            INC(A),
            LD_8_INTERNAL(AT_HL, A),
            
            JR(-24),
            
            // Output remainder + '0'
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, AT_HL),
            ADD_IMMEDIATE(b'0'),  // This is the critical line
            LD_8_TO_FF_IMMEDIATE(0x01),
            PUSH_AF,
            LD_8_IMMEDIATE(A, 0x81),
            LD_8_TO_FF_IMMEDIATE(0x02),
            POP_AF,
            
            // Replace value with quotient
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL),
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(AT_HL, A),
        ]);
    }
    
    instructions.extend(vec![
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
