// Simplest possible 32-bit Z85 encoder
// Just test division of 210 (0x000000D2) → should give remainder sequence that encodes to "0002E"

use zerodmg_codes::instruction::prelude::*;
use zerodmg_codes::instruction::FlagCondition;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-32bit-simple.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-32bit-simple.gb");
    println!("Test: 210 (0x000000D2) should output remainders that encode to '0002E'");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'3', b'2', b'S', b'I', b'M', 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x0148 { rom.push(0); }
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
    
    vec![
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Store test value: 210 = 0xD2 at byte 0, rest zeros
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_IMMEDIATE(A, 0xD2), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A), INC_16(HL),
        LD_8_IMMEDIATE(A, 0x00), LD_8_INTERNAL(AT_HL, A),
        
        // Output "Start: "
        LD_8_IMMEDIATE(A, b'S'), CALL(serial_out),
        LD_8_IMMEDIATE(A, b't'), CALL(serial_out),
        LD_8_IMMEDIATE(A, b'a'), CALL(serial_out),
        LD_8_IMMEDIATE(A, b'r'), CALL(serial_out),
        LD_8_IMMEDIATE(A, b't'), CALL(serial_out),
        LD_8_IMMEDIATE(A, b':'), CALL(serial_out),
        LD_8_IMMEDIATE(A, b' '), CALL(serial_out),
        
        // Divide once and output remainder
        CALL(div_32_by_85),
        // A has remainder (0-84)
        // Convert to ASCII digit by adding '0' (works for 0-9)
        ADD_IMMEDIATE(b'0'),
        CALL(serial_out),
        LD_8_IMMEDIATE(A, b' '), CALL(serial_out),
        
        // Divide again
        CALL(div_32_by_85),
        ADD_IMMEDIATE(b'0'),
        CALL(serial_out),
        LD_8_IMMEDIATE(A, b' '), CALL(serial_out),
        
        // Divide again
        CALL(div_32_by_85),
        ADD_IMMEDIATE(b'0'),
        CALL(serial_out),
        LD_8_IMMEDIATE(A, b' '), CALL(serial_out),
        
        // Divide again
        CALL(div_32_by_85),
        ADD_IMMEDIATE(b'0'),
        CALL(serial_out),
        LD_8_IMMEDIATE(A, b' '), CALL(serial_out),
        
        // Divide last time
        CALL(div_32_by_85),
        ADD_IMMEDIATE(b'0'),
        CALL(serial_out),
        
        // Newline
        LD_8_IMMEDIATE(A, b'\n'), CALL(serial_out),
        
        HALT,
        JR(-1),
    ]
    .into_iter()
    .chain(serial_out_function())
    .chain(div_32_by_85_function())
    .collect()
}

fn serial_out_function() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use FlagCondition::*;
    
    vec![
        // Function: serial_out (A = byte to send)
        LD_8_TO_FF_IMMEDIATE(0x01), // Write to SB (0xFF01)
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02), // Write to SC (0xFF02) - start transfer
        RET,
    ]
}

fn div_32_by_85_function() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    use FlagCondition::*;
    
    // Divide 32-bit value at 0xC100-0xC103 by 85
    // Return remainder in A, quotient left in memory
    vec![
        // Check if value < 85 (check bytes 3,2,1 are all 0, then byte 0 < 85)
        LD_16_IMMEDIATE(HL, 0xC103),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 15), // Byte 3 non-zero
        
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 10), // Byte 2 non-zero
        
        DEC_16(HL),
        LD_8_INTERNAL(A, AT_HL), OR(A), JR_IF(if_NZ, 5), // Byte 1 non-zero
        
        DEC_16(HL), // HL now = 0xC100
        LD_8_INTERNAL(A, AT_HL),
        CP_IMMEDIATE(85),
        JR_IF(if_C, 26), // Less than 85, we're done
        
        // Value >= 85, subtract using SBC chain
        // First byte: SUB (sets carry if borrow)
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        SUB_IMMEDIATE(85),
        LD_8_INTERNAL(AT_HL, A),
        
        // Remaining bytes: SBC 0 (propagate borrow)
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
        
        // Loop back to check again
        JR(-45),
        
        // Done: remainder is in byte 0
        LD_16_IMMEDIATE(HL, 0xC100),
        LD_8_INTERNAL(A, AT_HL),
        RET,
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
