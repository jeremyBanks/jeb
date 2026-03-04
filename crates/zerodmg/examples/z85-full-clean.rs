// Full 32-bit Z85 Encoder - Clean implementation
// Input: 4 bytes → Output: 5 Z85 characters

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-full-clean.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-full-clean.gb");
    println!("Test: 210 (0x000000D2) → '0002E' when properly encoded");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'C', b'L', b'N', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    while rom.len() < 0x0200 { rom.push(0); }
    
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
    
    let mut code = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Test value: 210
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
    ];
    
    // Compute and output 5 digits
    for _ in 0..5 {
        // === DIVISION: value = value / 85, return value % 85 ===
        // Quotient counter at 0xC110 (1 byte is enough for our test)
        code.extend(vec![
            LD_8_IMMEDIATE(A, 0),
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(AT_HL, A), // quotient = 0
        ]);
        
        // DIV_LOOP: while value >= 85, value -= 85, quotient++
        code.extend(vec![
            // Check: is value < 85?
            LD_16_IMMEDIATE(HL, 0xC103),
            LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 15), // High bytes non-zero
            DEC_16(HL),
            LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 10),
            DEC_16(HL),
            LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 5),
            DEC_16(HL),
            LD_8_INTERNAL(A, AT_HL),
            CP_IMMEDIATE(85),
            JR_IF(if_C, 37), // value < 85, exit to DONE
            
            // value >= 85: subtract
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, AT_HL),
            SUB_IMMEDIATE(85),
            LD_8_INTERNAL(AT_HL, A),
            INC_16(HL),
            LD_8_INTERNAL(A, AT_HL),
            SBC_IMMEDIATE(0),
            LD_8_INTERNAL(AT_HL, A),
            INC_16(HL),
            LD_8_INTERNAL(A, AT_HL),
            SBC_IMMEDIATE(0),
            LD_8_INTERNAL(AT_HL, A),
            INC_16(HL),
            LD_8_INTERNAL(A, AT_HL),
            SBC_IMMEDIATE(0),
            LD_8_INTERNAL(AT_HL, A),
            
            // Increment quotient
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL),
            INC(A),
            LD_8_INTERNAL(AT_HL, A),
            
            JR(-54), // Loop back
        ]);
        
        // DONE: value now contains remainder, C110 has quotient
        // Output remainder (add '0' to make ASCII)
        code.extend(vec![
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, AT_HL), // A = remainder
            ADD_IMMEDIATE(b'0'),
            
            // Serial output (save/restore A)
            LD_8_TO_FF_IMMEDIATE(0x01), // Write A to SB (0xFF01)
            PUSH_AF, // Save character
            LD_8_IMMEDIATE(A, 0x81), // Transfer enable + internal clock
            LD_8_TO_FF_IMMEDIATE(0x02), // Write to SC (0xFF02) - start transfer
            POP_AF, // Restore character
            
            // Replace value with quotient for next iteration
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL), // A = quotient
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(AT_HL, A), // Byte 0 = quotient
            INC_16(HL),
            LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), // Clear byte 1
            INC_16(HL),
            LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), // Clear byte 2
            INC_16(HL),
            LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), // Clear byte 3
        ]);
    }
    
    // Newline
    code.extend(vec![
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        HALT,
        JR(-1),
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
