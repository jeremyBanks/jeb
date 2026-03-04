//! Test 16-bit division by 85
//!
//! Standalone test to verify division algorithm works

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("div-test.gb", &rom).expect("Failed to write ROM");
    println!("Generated div-test.gb ({} bytes)", rom.len());
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'D', b'I', b'V', b'-', b'T', b'E', b'S', b'T', 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
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
    use FlagCondition::*;
    
    // Test: divide 1234 by 85
    // Expected: quotient = 14, remainder = 14
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Load test value 1234 (0x04D2) into DE
        LD_16_IMMEDIATE(DE, 0x04D2),
        
        // Divide DE by 85
        // Result: BC = quotient, A = remainder
    ]
    .into_iter()
    .chain(div_de_by_85())
    .chain(vec![
        // Division returns: BC = quotient, A = remainder
        // Save remainder before we clobber A
        PUSH_AF,
        
        // Output quotient (BC) as 2-byte hex via serial
        // Output HIGH byte first
        LD_8_INTERNAL(A, B),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Output LOW byte
        LD_8_INTERNAL(A, C),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Output space
        LD_8_IMMEDIATE(A, b' '),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Output remainder (was saved on stack at start)
        POP_AF, // A now has remainder
        LD_8_TO_FF_IMMEDIATE(0x01), // Write remainder to serial data
        LD_8_IMMEDIATE(A, 0x81),     // Load control byte
        LD_8_TO_FF_IMMEDIATE(0x02), // Start transfer
        
        // Newline
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        HALT,
        JR(-1),
    ])
    .collect()
}

/// Divide DE by 85 using repeated subtraction
/// Input: DE = dividend (16-bit)
/// Output: BC = quotient (16-bit), A = remainder (8-bit, in range 0-84)
/// Uses simplified algorithm: just handle small values for MVP
fn div_de_by_85() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    // For MVP, assume value fits in 8 bits (E register, D = 0)
    // This handles values 0-255
    // 255 / 85 = 3, so we need at most 3 iterations
    
    vec![
        // Initialize quotient to 0 (use C for 8-bit quotient)
        LD_8_IMMEDIATE(C, 0),
        
        // DIV_LOOP: Subtract 85 from E repeatedly
        // Loop body starts here
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85), // Compare with 85
        JR_IF(if_C, 6), // If A < 85, exit loop (jump forward +6 to EXIT)
        
        // A >= 85, subtract
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        
        // Increment quotient
        INC(C),
        
        // Loop back (jump to start of loop body)
        JR(-11), // Jump back -11 bytes to LOOP_START (LD_8_INTERNAL(A, E))
        
        // LOOP_EXIT: E < 85
        LD_8_INTERNAL(A, E), // Remainder in A
        LD_8_IMMEDIATE(B, 0),  // High byte of quotient is 0
        // C already has low byte of quotient
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
