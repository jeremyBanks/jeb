//! 32-bit division using the proven approach from z85-simple
//! Store value in memory, use memory-based quotient counter

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-32bit-simple.gb", &rom).expect("Failed");
    println!("Generated z85-32bit-simple.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[b'3',b'2',b'S',b'I',b'M',0,0,0,0,0,0,0,0,0,0,0]);
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    while rom.len() < 0x0260 { rom.push(0); }
    for inst in game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    while rom.len() < 32768 { rom.push(0); }
    rom
}

fn game_code() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Store value 210 at 0xC100-0xC103 (4 bytes, little-endian)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 210), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A),
        
        // Quotient counter at 0xC120
        LD_16_IMMEDIATE(HL, 0xC120),
        LD_8_IMMEDIATE(A, 0),
        LD_8_INTERNAL(AT_HL, A),
        
        // LOOP: while value >= 85
        // Check bytes 3,2,1,0
        LD_16_IMMEDIATE(HL, 0xC103), // byte 3
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 11), // Jump to subtract
        DEC_16(HL), // byte 2
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 7),
        DEC_16(HL), // byte 1
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 3),
        DEC_16(HL), // byte 0
        LD_8_INTERNAL(A, AT_HL),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 31), // Exit if < 85 (jump to output)
        
        // Subtract 85 from 4-byte value
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL), SUB_IMMEDIATE(85), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_INTERNAL(A, AT_HL), SBC_IMMEDIATE(0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_INTERNAL(A, AT_HL), SBC_IMMEDIATE(0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_INTERNAL(A, AT_HL), SBC_IMMEDIATE(0), LD_8_INTERNAL(AT_HL, A),
        
        // Increment quotient
        LD_16_IMMEDIATE(HL, 0xC120),
        LD_8_INTERNAL(A, AT_HL), INC(A), LD_8_INTERNAL(AT_HL, A),
        
        // Loop back to LD_16_IMMEDIATE(HL, 0xC103)
        JR(-54), // 52 bytes from end of JR to loop start + 2 for PC
        
        // Output remainder (byte 0) and quotient
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        LD_16_IMMEDIATE(HL, 0xC120),
        LD_8_INTERNAL(A, AT_HL),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
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
