// 32-bit Z85 - cleaner approach based on 16-bit version

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-32-v2.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-32-v2.gb");
    println!("Test: 210 → '( 2 0 0 0' where ( = chr(40)");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'V', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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

fn div_and_output() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        // Save current value remainder before computing quotient
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        LD_16_IMMEDIATE(HL, 0xC120), // Temp location
        LD_8_INTERNAL(AT_HL, A), // Save byte 0
        
        // Initialize quotient to 0
        LD_8_IMMEDIATE(A, 0),
        LD_16_IMMEDIATE(HL, 0xC110),
        LD_8_INTERNAL(AT_HL, A),
        
        // Subtract loop
        // LOOP_START (target for JR):
        LD_16_IMMEDIATE(HL, 0xC103),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 15),
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 10),
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 5),
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 37), // Exit to OUTPUT
        
        // Subtract 85
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
        
        // Inc quotient
        LD_16_IMMEDIATE(HL, 0xC110),
        LD_8_INTERNAL(A, AT_HL),
        INC(A),
        LD_8_INTERNAL(AT_HL, A),
        
        JR(-54), // Back to LOOP_START
        
        // OUTPUT: Get remainder from before division
        LD_16_IMMEDIATE(HL, 0xC120),
        LD_8_INTERNAL(A, AT_HL), // A = original byte 0
        
        // Now we need: remainder = original_value % 85
        // But we've divided value by 85, so:
        // original = quotient * 85 + remainder
        // remainder = original - quotient * 85
        
        // Actually this is wrong. Let me rethink...
    ]
}

fn game_code() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Store 210 at 0xC100
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
        
        // Just output "OK\n" for now to test basic serial
        LD_8_IMMEDIATE(A, b'O'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        LD_8_IMMEDIATE(A, b'K'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
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
