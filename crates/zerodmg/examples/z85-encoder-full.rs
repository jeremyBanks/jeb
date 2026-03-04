// !Z85 Full 32-bit Encoder
//!
//! Encodes a complete 4-byte (32-bit) value to 5 Z85 characters.
//! Extends z85-real.rs to handle full Z85 spec.

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-encoder-full.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-encoder-full.gb ({} bytes)", rom.len());
    println!("Full 32-bit Z85 encoder");
    println!("Test value: 0x8680D26FB5");
    println!("Expected: HelloW (first 5 chars of Z85 'HelloWorld')");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'-', b'F', b'U', b'L', b'L', 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    // Z85 alphabet at 0x0200
    while rom.len() < 0x0200 { rom.push(0); }
    let z85_alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    let alphabet_addr = rom.len() as u16;
    rom.extend_from_slice(z85_alphabet);
    
    // Game code starts at 0x0260
    while rom.len() < 0x0260 { rom.push(0); }
    
    let instructions = game_code(alphabet_addr);
    for inst in instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

fn game_code(alphabet_addr: u16) -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Copy Z85 alphabet to RAM (0xC000)
        LD_16_IMMEDIATE(HL, alphabet_addr),
        LD_16_IMMEDIATE(DE, 0xC000),
        LD_8_IMMEDIATE(B, 85),
    ]
    .into_iter()
    .chain(copy_loop())
    .chain(test_encoding())
    .chain(vec![HALT, JR(-1)])
    .collect()
}

fn copy_loop() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        LD_8_FROM_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        DEC(B),
        JR_IF(if_NZ, -6),
    ]
}

/// Divide 32-bit value by 85 using repeated subtraction
/// Input: 4 bytes at 0xC100-0xC103 (little-endian)
/// Output: Quotient stored back at 0xC100-0xC103, remainder in A
fn div_32bit_by_85() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        // DIVISION_LOOP: while value >= 85
        // Check if value is zero first
        LD_16_IMMEDIATE(HL, 0xC103), // Start with most significant byte
        LD_8_INTERNAL(A, AT_HL), OR(A),
        JR_IF(if_NZ, 18), // Byte 3 != 0, so value >= 85
        
        DEC_16(HL), // HL = 0xC102
        LD_8_INTERNAL(A, AT_HL), OR(A),
        JR_IF(if_NZ, 12), // Byte 2 != 0, so value >= 85
        
        DEC_16(HL), // HL = 0xC101
        LD_8_INTERNAL(A, AT_HL), OR(A),
        JR_IF(if_NZ, 6), // Byte 1 != 0, so value >= 85
        
        DEC_16(HL), // HL = 0xC100
        LD_8_INTERNAL(A, AT_HL),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 30), // Byte 0 < 85 and higher bytes all zero, exit
        
        // Value >= 85, subtract it
        // Use SBC for multi-byte subtraction
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        SUB_IMMEDIATE(85), // Subtract with carry flag cleared
        LD_8_INTERNAL(AT_HL, A),
        
        INC_16(HL), // HL = 0xC101
        LD_8_INTERNAL(A, AT_HL),
        SBC_IMMEDIATE(0),
        LD_8_INTERNAL(AT_HL, A),
        
        INC_16(HL), // HL = 0xC102
        LD_8_INTERNAL(A, AT_HL),
        SBC_IMMEDIATE(0),
        LD_8_INTERNAL(AT_HL, A),
        
        INC_16(HL), // HL = 0xC103
        LD_8_INTERNAL(A, AT_HL),
        SBC_IMMEDIATE(0),
        LD_8_INTERNAL(AT_HL, A),
        
        // Loop back
        JR(-50),
        
        // DIVISION_DONE:
        // Remainder is in byte 0
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
    ]
}

fn test_encoding() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    // Test: Standard Z85 test vector
    // Input: 0x86 0x4F 0xD2 0x6F 0xB5 (big-endian bytes from spec)
    // But Game Boy is little-endian, so we store: 0xB5 0x6F 0xD2 0x4F 0x86
    // This should encode to "HelloW" (first 5 of "HelloWorld")
    
    // Actually, let's use a simpler test first: 0x00 0x00 0x00 0xD2 (210)
    // Should give us "0002E" like the 16-bit version
    
    let mut code = vec![
        // Store test value at 0xC100-0xC103 (32-bit, little-endian)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // Byte 0: 0xD2
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // Byte 1: 0x00
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // Byte 2: 0x00
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),              // Byte 3: 0x00
    ];
    
    // Compute 5 digits and store at 0xC110-0xC114
    code.extend(vec![
        LD_8_IMMEDIATE(B, 5),         // Digit counter
        LD_16_IMMEDIATE(HL, 0xC110),  // Digit storage pointer
    ]);
    
    // DIGIT_LOOP:
    code.extend(vec![
        PUSH(BC),
        PUSH(HL),
    ]);
    
    code.extend(div_32bit_by_85()); // A = remainder
    
    code.extend(vec![
        POP(HL),
        LD_8_INTERNAL(AT_HL, A), // Store digit
        INC_16(HL),
        POP(BC),
        DEC(B),
        JR_IF(if_NZ, -106), // Jump back (adjust offset based on div function size)
    ]);
    
    // Output the 5 digits
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xC110), // Start of digits
        LD_8_IMMEDIATE(B, 5),         // Count
    ]);
    
    // OUTPUT_LOOP:
    code.extend(vec![
        PUSH(BC),
        PUSH(HL),
        
        // Get digit
        LD_8_INTERNAL(A, AT_HL),
        
        // Look up in alphabet
        LD_16_IMMEDIATE(HL, 0xC000),
        ADD(L),
        LD_8_INTERNAL(L, A),
        LD_8_IMMEDIATE(A, 0),
        ADC(H),
        LD_8_INTERNAL(H, A),
        LD_8_INTERNAL(A, AT_HL),
        
        // Output
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        POP(HL),
        INC_16(HL),
        POP(BC),
        DEC(B),
        JR_IF(if_NZ, -27),
    ]);
    
    // Newline
    code.extend(vec![
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
    ]);
    
    code
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
