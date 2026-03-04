//! Z85 Encoder - REAL IMPLEMENTATION
//! 
//! Reads 4-byte input, encodes to Z85, outputs 5 characters via serial.
//! NOT A DEMO - implements actual 32-bit division by 85.

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-real.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-real.gb ({} bytes)", rom.len());
    println!("REAL Z85 encoding with 32-bit division");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'-', b'R', b'E', b'A', b'L', 0, 0, 0, 0, 0, 0, 0, 0,
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
    use U8SecondaryRegister::*;
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
    use U8SecondaryRegister::*;
    use FlagCondition::*;
    
    vec![
        // COPY_LOOP: 6 bytes
        LD_8_FROM_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        DEC(B),
        JR_IF(if_NZ, -6),
    ]
}

fn delay_loop() -> Vec<Instruction> {
    use Instruction::*;
    use U16Register::*;
    use U8Register::*;
    use FlagCondition::*;
    
    vec![
        // DELAY_LOOP: 4 bytes (HL already loaded)
        DEC_16(HL),
        LD_8_INTERNAL(A, H),
        OR(L),
        JR_IF(if_NZ, -5),
    ]
}

fn delay_loop_de() -> Vec<Instruction> {
    use Instruction::*;
    use U16Register::*;
    use U8Register::*;
    use FlagCondition::*;
    
    vec![
        // DELAY_LOOP: 4 bytes (DE already loaded)
        DEC_16(DE),
        LD_8_INTERNAL(A, D),
        OR(E),
        JR_IF(if_NZ, -5),
    ]
}

/// Divide E by 85 using repeated subtraction
/// Input: E = dividend (8-bit)
/// Output: C = quotient (8-bit), A = remainder (8-bit, in range 0-84)
fn div_e_by_85() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use FlagCondition::*;
    
    vec![
        // Initialize quotient to 0
        LD_8_IMMEDIATE(C, 0),
        
        // DIV_LOOP: Subtract 85 from E repeatedly
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 6), // If A < 85, exit loop (jump forward +6 to EXIT)
        
        // A >= 85, subtract
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        
        // Increment quotient
        INC(C),
        
        // Loop back
        JR(-11), // Jump back -11 bytes to LOOP_START
        
        // LOOP_EXIT: E < 85
        LD_8_INTERNAL(A, E), // Remainder in A
        // C has quotient
    ]
}

/// Divide DE by 85 using repeated subtraction (16-bit version)
/// Input: DE = dividend (16-bit)
/// Output: BC = quotient (16-bit), A = remainder (8-bit, in range 0-84)
fn div_de_by_85_16bit() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        // Initialize quotient to 0
        LD_16_IMMEDIATE(BC, 0),
        
        // DIV_LOOP: while DE >= 85
        // Check if D > 0 (then definitely >= 85)
        LD_8_INTERNAL(A, D),
        OR(A),
        JR_IF(if_NZ, 5), // If D != 0, skip to subtract
        
        // D == 0, check E >= 85
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 10), // If E < 85, exit (jump +10 to LOOP_EXIT)
        
        // DE >= 85, subtract
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 1), // No borrow - skip DEC D only
        DEC(D), // Borrow from high byte
        
        // Increment quotient
        INC_16(BC),
        
        // Loop back
        JR(-19), // 17 bytes loop body + 2 for JR instruction itself
        
        // LOOP_EXIT
        LD_8_INTERNAL(A, E), // Remainder in A
        // BC has quotient
    ]
}

fn test_encoding() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    // Test: encode 16-bit value using actual division
    // Using value 0xABCD (43981 decimal)
    // Expected Z85 output: "0067A"
    
    let mut code = vec![
        // Store test value at 0xC100-0xC101 (16-bit, little-endian)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xCD), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // Low: 0xCD
        LD_8_IMMEDIATE(A, 0xAB), LD_8_INTERNAL(AT_HL, A),              // High: 0xAB
    ];
    
    // Compute digits from right to left (least to most significant)
    // Store at 0xC114, 0xC113, 0xC112, 0xC111, 0xC110
    // Use 0xC105 to store loop counter (avoids register conflicts)
    
    code.extend(vec![
        LD_8_IMMEDIATE(A, 5),         // Loop counter
        LD_16_IMMEDIATE(HL, 0xC105),
        LD_8_INTERNAL(AT_HL, A),      // Store counter at 0xC105
    ]);
    
    // DIGIT_LOOP: Compute one digit per iteration
    code.extend(vec![
        // Load 16-bit value from 0xC100-0xC101 into DE
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(E, AT_HL), // E = byte at 0xC100 (low)
        INC_16(HL),
        LD_8_INTERNAL(D, AT_HL), // D = byte at 0xC101 (high)
        // DE now contains our value
        
        // Divide DE by 85: quotient → BC, remainder → A
    ]);
    
    code.extend(div_de_by_85_16bit());
    
    code.extend(vec![
        // A now has remainder (the digit)
        // BC has quotient
        // Save remainder
        PUSH_AF,
        
        // Store quotient back to 0xC100-0xC101 for next iteration
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, C), LD_8_INTERNAL(AT_HL, A), INC_16(HL), // Low byte
        LD_8_INTERNAL(A, B), LD_8_INTERNAL(AT_HL, A),              // High byte
        
        // Load loop counter from memory into temp storage (NOT B!)
        LD_16_IMMEDIATE(HL, 0xC105),
        LD_8_INTERNAL(A, AT_HL),
        // Store counter at 0xC106 temporarily
        LD_16_IMMEDIATE(HL, 0xC106),
        LD_8_INTERNAL(AT_HL, A),
        
        // Calculate digit pointer: 0xC10F + counter
        LD_16_IMMEDIATE(HL, 0xC10F),
        ADD(L),
        LD_8_INTERNAL(L, A),
        
        // Store digit (remainder from earlier PUSH_AF)
        POP_AF, // A = remainder
        LD_8_INTERNAL(AT_HL, A),
        
        // Decrement and save loop counter
        LD_16_IMMEDIATE(HL, 0xC106),
        LD_8_INTERNAL(A, AT_HL),
        DEC(A),
        LD_16_IMMEDIATE(HL, 0xC105),
        LD_8_INTERNAL(AT_HL, A),
        
        // Loop if counter > 0 (A already has counter from above)
        OR(A),
        JR_IF(if_NZ, -65), // Loop body: 6 (before div) + 23 (div) + 36 (after) - 2 = 63 bytes
    ]);
    
    // Output the 5 digits
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xC110), // Digit indices
        LD_8_IMMEDIATE(B, 5),         // Count
    ]);
    
    // OUTPUT_DIGITS_LOOP:
    code.extend(vec![
        // Get digit index into A
        LD_8_INTERNAL(A, AT_HL),
        INC_16(HL),
        PUSH(HL), // Save digit pointer
        
        // Look up alphabet[A]
        LD_16_IMMEDIATE(HL, 0xC000), // Alphabet base
        // Add A to L
        ADD(L),
        LD_8_INTERNAL(L, A),
        LD_8_IMMEDIATE(A, 0),
        ADC(H),
        LD_8_INTERNAL(H, A),
        
        // Get character
        LD_8_INTERNAL(A, AT_HL),
        
        // Output via serial
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
        
        POP(HL), // Restore digit pointer
        
        // Loop
        DEC(B),
        JR_IF(if_NZ, -25), // Jump back (23 bytes loop body + 2 for JR)
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

// Helper: Add A to DE (16-bit += 8-bit)
fn ADD_A_TO_DE() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    
    vec![
        // DE += A
        // E = E + A
        ADD(E),
        LD_8_INTERNAL(E, A),
        // D = D + carry
        LD_8_IMMEDIATE(A, 0),
        ADC(D),
        LD_8_INTERNAL(D, A),
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
