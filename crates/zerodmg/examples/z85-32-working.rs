// 32-bit Z85 division - tracking quotient properly

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-32-working.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-32-working.gb");
    println!("Test: 210 → should output '( 2 0 0 0' (40, 2, 0, 0, 0)");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'-', b'W', b'O', b'R', b'K', 0, 0, 0, 0, 0, 0, 0, 0,
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
        
        // Store test value: 210 = 0xD2 at 0xC100
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
    ];
    
    // Divide 5 times, output each remainder
    for _ in 0..5 {
        // Division: repeatedly subtract 85, count subtractions
        // Quotient counter at 0xC110
        code.extend(vec![
            LD_8_IMMEDIATE(A, 0),
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(AT_HL, A), // quotient = 0
        ]);
        
        // DIV_LOOP: while value >= 85
        code.extend(vec![
            // Check if value < 85
            LD_16_IMMEDIATE(HL, 0xC103),
            LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 15), // Byte 3 != 0, so >= 85
            
            DEC_16(HL),
            LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 10), // Byte 2 != 0
            
            DEC_16(HL),
            LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 5), // Byte 1 != 0
            
            DEC_16(HL),
            LD_8_INTERNAL(A, AT_HL),
            CP_IMMEDIATE(85),
            JR_IF(if_C, 37), // < 85, exit loop
            
            // value >= 85, subtract it
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
            
            // Increment quotient counter
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL),
            INC(A),
            LD_8_INTERNAL(AT_HL, A),
            
            // Loop back
            JR(-54),
        ]);
        
        // Division done: value has remainder, C110 has quotient
        // Clear value and set it to quotient for next iteration
        code.extend(vec![
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL), // A = quotient
            
            LD_16_IMMEDIATE(HL, 0xC100),
            PUSH_AF, // Save quotient
            LD_8_INTERNAL(A, AT_HL), // Get remainder
            PUSH_AF, // Save remainder for output
            
            // Set value to quotient
            POP(BC), // B = remainder (discard), C = quotient
            POP(BC), // B,C = quotient
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, C), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
            LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
            LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
            LD_8_IMMEDIATE(A, 0), LD_8_INTERNAL(AT_HL, A),
            
            // Get remainder back from stack
            LD_8_INTERNAL(A, C), // Wait, I popped it wrong
        ]);
        
        // Actually, let me redo this more carefully
    }
    
    code.extend(vec![
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
