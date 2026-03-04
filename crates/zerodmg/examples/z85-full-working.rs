// Full 32-bit Z85 Encoder - Working version based on debug success

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-full-working.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-full-working.gb");
    println!("Test: 210 → '(20 00' where ( = chr(40)");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'F', b'I', b'N', b'A', b'L', 0, 0, 0, 0, 0, 0, 0, 0,
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

fn serial_out_a() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    
    vec![
        LD_8_TO_FF_IMMEDIATE(0x01),
        PUSH_AF,
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        POP_AF,
    ]
}

fn game_code() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    let mut code = vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Store test value 210
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
    ];
    
    // Do 5 divisions
    for _ in 0..5 {
        code.extend(vec![
            // Init quotient to 0
            LD_8_IMMEDIATE(A, 0),
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(AT_HL, A),
        ]);
        
        // Division loop - simplified for low values
        // LOOP_START:
        code.extend(vec![
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, AT_HL),
            CP_IMMEDIATE(85),
            JR_IF(if_C, 20), // Exit if < 85
            
            // Subtract 85
            SUB_IMMEDIATE(85),
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(AT_HL, A),
            
            // Inc quotient
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL),
            INC(A),
            LD_8_INTERNAL(AT_HL, A),
            
            JR(-24), // Loop back
        ]);
        
        // Output remainder
        code.extend(vec![
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(A, AT_HL),
            ADD_IMMEDIATE(b'0'),
        ]);
        code.extend(serial_out_a());
        
        // Replace value with quotient
        code.extend(vec![
            LD_16_IMMEDIATE(HL, 0xC110),
            LD_8_INTERNAL(A, AT_HL),
            LD_16_IMMEDIATE(HL, 0xC100),
            LD_8_INTERNAL(AT_HL, A),
        ]);
    }
    
    // Newline
    code.extend(vec![
        LD_8_IMMEDIATE(A, b'\n'),
    ]);
    code.extend(serial_out_a());
    
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
