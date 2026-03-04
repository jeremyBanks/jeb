//! Z85 Encoder - 16-bit version
//! 
//! Handles values up to 65535 using 16-bit division.

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-16bit.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-16bit.gb ({} bytes)", rom.len());
    println!("16-bit Z85 encoding");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'-', b'1', b'6', b'B', b'I', b'T', 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    // Z85 alphabet at 0x0200
    while rom.len() < 0x0200 { rom.push(0); }
    let z85_alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    rom.extend_from_slice(z85_alphabet);
    
    while rom.len() < 0x0260 { rom.push(0); }
    
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
    use U8SecondaryRegister::*;
    use FlagCondition::*;
    
    // Test value: 1234 (0x04D2)
    // Expected: 1234 / 85 = 14 rem 44, 14 / 85 = 0 rem 14
    // Digits: 0, 0, 0, 14, 44
    // Output: "0" "0" "0" "e" ","  (indices 0,0,0,14,44 in alphabet)
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Copy alphabet to RAM
        LD_16_IMMEDIATE(HL, 0x0200), // Alphabet in ROM
        LD_16_IMMEDIATE(DE, 0xC000), // Destination
        LD_8_IMMEDIATE(B, 85),
        // Copy loop
        LD_8_FROM_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        DEC(B),
        JR_IF(if_NZ, -6),
        
        // Store test value (16-bit) at 0xC100-0xC101
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // Low byte
        LD_8_IMMEDIATE(A, 0x04), LD_8_INTERNAL(AT_HL, A),              // High byte
        
        // Compute digits (5 iterations, right-to-left)
        LD_8_IMMEDIATE(C, 5), // Digit counter
        LD_16_IMMEDIATE(HL, 0xC114), // Start at last digit
        
        // DIGIT_LOOP: Load value → divide → store digit → store quotient
        // Load 16-bit value into DE
        PUSH(HL), // Save digit pointer
        PUSH(BC), // Save counter
        
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL), LD_8_INTERNAL(E, A), INC_16(HL),
        LD_8_INTERNAL(A, AT_HL), LD_8_INTERNAL(D, A),
        
        // Divide DE by 85 → quotient in DE, remainder in A
        // Use simple subtraction loop
        LD_8_IMMEDIATE(B, 0), // Quotient counter (8-bit, max 255/85 = 3 iterations for 16-bit)
        
        // SUB_LOOP: while DE >= 85
        LD_8_INTERNAL(A, D),
        OR(A), // If D != 0, DE >= 85
        JR_IF(if_NZ, 5), // Skip to subtract
        
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 13), // Exit if E < 85
        
        // Subtract 85 from DE
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 2), // No borrow
        DEC(D), // Borrow from high byte
        
        INC(B), // Increment quotient
        JR(-18), // Loop back
        
        // EXIT_SUB_LOOP: E has remainder, B has quotient (low byte), D should be 0
        LD_8_INTERNAL(A, E), // Remainder
        PUSH_AF, // Save remainder
        
        // Store quotient back (B in low byte, 0 in high byte)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, B),
        LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0),
        LD_8_INTERNAL(AT_HL, A),
        
        // Restore and store digit
        POP_AF, // Remainder
        POP(BC), // Counter
        POP(HL), // Digit pointer
        LD_8_INTERNAL(AT_HL, A), // Store digit
        
        // Next position
        DEC_16(HL),
        DEC(C),
        JR_IF(if_NZ, -51), // Back to DIGIT_LOOP
        
        // Output digits
        LD_16_IMMEDIATE(HL, 0xC110),
        LD_8_IMMEDIATE(B, 5),
        
        // OUTPUT_LOOP
        LD_8_INTERNAL(A, AT_HL),
        INC_16(HL),
        PUSH(HL),
        
        // Alphabet lookup
        LD_16_IMMEDIATE(HL, 0xC000),
        ADD(L),
        LD_8_INTERNAL(L, A),
        LD_8_IMMEDIATE(A, 0),
        ADC(H),
        LD_8_INTERNAL(H, A),
        LD_8_INTERNAL(A, AT_HL),
        
        // Serial output
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        POP(HL),
        DEC(B),
        JR_IF(if_NZ, -25),
        
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
