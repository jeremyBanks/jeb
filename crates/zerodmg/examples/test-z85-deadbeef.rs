//! Test Z85 with 0xDEADBEEF (a proper 32-bit value)

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("test-z85-deadbeef.gb", &rom).expect("Failed to write ROM");
    println!("Generated test-z85-deadbeef.gb");
    println!("Test value: 0xBEEF (48879 decimal)");
    
    // Calculate expected Z85 output
    let val = 0xBEEF_u16 as u32;
    let mut v = val;
    let mut digits = Vec::new();
    for _ in 0..5 {
        digits.push((v % 85) as u8);
        v /= 85;
    }
    digits.reverse();
    
    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    let expected: String = digits.iter().map(|&d| alphabet[d as usize] as char).collect();
    println!("Expected output: {}", expected);
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'B', b'E', b'E', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    // Z85 alphabet at 0x0200
    while rom.len() < 0x0200 { rom.push(0); }
    let z85_alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    rom.extend_from_slice(z85_alphabet);
    
    // Copy alphabet to RAM
    while rom.len() < 0x0260 { rom.push(0); }
    
    let mut code = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Copy alphabet to 0xC000
        LD_16_IMMEDIATE(HL, 0x0200),
        LD_16_IMMEDIATE(DE, 0xC000),
        LD_8_IMMEDIATE(B, 85),
    ];
    
    code.extend(copy_loop());
    
    // Store test value 0xBEEF at 0xC100 (little-endian, 16-bit)
    code.extend(vec![
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xEF), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0xBE), LD_8_INTERNAL(AT_HL, A),
        
        // Initialize loop counter
        LD_8_IMMEDIATE(A, 5),
        LD_16_IMMEDIATE(HL, 0xC105),
        LD_8_INTERNAL(AT_HL, A),
    ]);
    
    // DIGIT_LOOP
    code.extend(encode_loop());
    
    // Output digits
    code.extend(output_digits());
    
    code.extend(vec![
        HALT,
        JR(-1),
    ]);
    
    for inst in code {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

fn copy_loop() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use U8SecondaryRegister::*;
    use FlagCondition::*;
    
    vec![
        LD_8_FROM_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_DE),
        INC_16(DE),
        DEC(B),
        JR_IF(if_NZ, -6),
    ]
}

fn encode_loop() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        // Load 32-bit value from 0xC100 into DE (low 16 bits)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(E, AT_HL),
        INC_16(HL),
        LD_8_INTERNAL(D, AT_HL),
        
        // Divide DE by 85
        LD_16_IMMEDIATE(BC, 0),
        LD_8_INTERNAL(A, D),
        OR(A),
        JR_IF(if_NZ, 5),
        LD_8_INTERNAL(A, E),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 10),
        LD_8_INTERNAL(A, E),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(E, A),
        JR_IF(if_NC, 1),  // Fixed!
        DEC(D),
        INC_16(BC),
        JR(-19),
        LD_8_INTERNAL(A, E),
        
        PUSH_AF,  // Save remainder
        
        // Store quotient back
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, C), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_INTERNAL(A, B), LD_8_INTERNAL(AT_HL, A),
        
        // Get loop counter, calculate digit address
        LD_16_IMMEDIATE(HL, 0xC105),
        LD_8_INTERNAL(A, AT_HL),
        LD_16_IMMEDIATE(HL, 0xC106),
        LD_8_INTERNAL(AT_HL, A),
        LD_16_IMMEDIATE(HL, 0xC10F),
        ADD(L),
        LD_8_INTERNAL(L, A),
        
        POP_AF,  // Remainder
        LD_8_INTERNAL(AT_HL, A),
        
        // Decrement counter
        LD_16_IMMEDIATE(HL, 0xC106),
        LD_8_INTERNAL(A, AT_HL),
        DEC(A),
        LD_16_IMMEDIATE(HL, 0xC105),
        LD_8_INTERNAL(AT_HL, A),
        
        OR(A),
        JR_IF(if_NZ, -65),
    ]
}

fn output_digits() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
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
        
        // Output
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
    ]
}

use Instruction::*;
use U8Register::*;
use U16Register::*;
use FlagCondition::*;
use U8SecondaryRegister::*;

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
