//! 32-bit division by 85 - the real deal
//! 
//! Test with known Z85 values to verify correctness

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-32bit-div.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-32bit-div.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[b'3',b'2',b'D',b'I',b'V',0,0,0,0,0,0,0,0,0,0,0]);
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
        
        // Test value: 210 (0x000000D2) - simple test
        // Expected: quotient=2, remainder=40
        // Store at 0xC100-0xC103 (little-endian)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // byte 0 (LSB)
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // byte 1
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // byte 2
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),              // byte 3 (MSB)
        
        // First, let's just test one division to verify 32-bit subtraction works
        // Load value into memory workspace at 0xC110-0xC113
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_16_IMMEDIATE(DE, 0xC110),
        LD_8_IMMEDIATE(B, 4), // Copy 4 bytes
        // Copy loop
        LD_8_INTERNAL(A, AT_HL), INC_16(HL),
        LD_8_TO_SECONDARY(AT_DE), INC_16(DE),
        DEC(B),
        JR_IF(if_NZ, -7),
        
        // Quotient counter at 0xC120
        LD_8_IMMEDIATE(A, 0),
        LD_16_IMMEDIATE(HL, 0xC120),
        LD_8_INTERNAL(AT_HL, A),
        
        // DIV_LOOP: while value >= 85, subtract 85 and increment counter
        // Compare 4-byte value at 0xC110 with 85
        // If any byte 1-3 is non-zero, value >= 85
        LD_16_IMMEDIATE(HL, 0xC113), // Start from MSB
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 16), // byte 3: jump to subtraction
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 11), // byte 2: jump to subtraction
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 6), // byte 1: jump to subtraction
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), // byte 0
        CP_IMMEDIATE(85),
        JR_IF(if_C, 30), // Exit if byte 0 < 85 (jump to output)
        
        // Value >= 85, subtract
        // Subtract 85 from byte 0, propagate borrow through bytes 1-3
        LD_16_IMMEDIATE(HL, 0xC110),
        LD_8_INTERNAL(A, AT_HL),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        
        // Propagate borrow through remaining bytes
        LD_8_INTERNAL(A, AT_HL), SBC_IMMEDIATE(0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_INTERNAL(A, AT_HL), SBC_IMMEDIATE(0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_INTERNAL(A, AT_HL), SBC_IMMEDIATE(0), LD_8_INTERNAL(AT_HL, A),
        
        // Increment quotient
        LD_16_IMMEDIATE(HL, 0xC120),
        LD_8_INTERNAL(A, AT_HL),
        INC(A),
        LD_8_INTERNAL(AT_HL, A),
        
        // Loop back to comparison start
        JR(-61), // 59 bytes (6 quotient + 30 subtract + 23 compare) + 2 for PC
        
        // Output: remainder (byte 0) and quotient
        LD_16_IMMEDIATE(HL, 0xC110),
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
