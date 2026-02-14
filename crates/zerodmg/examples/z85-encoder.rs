//! Z85 Encoder for Game Boy
//! 
//! Reads 4-byte chunks from serial port, encodes to Z85, outputs 5 characters.
//! 
//! Serial protocol:
//! - Input: Raw bytes (4 bytes per chunk, padded with zeros if needed)
//! - Output: Z85-encoded characters (5 chars per 4-byte chunk)

use zerodmg_codes::instruction::prelude::*;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-encoder.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-encoder.gb ({} bytes)", rom.len());
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    // Nintendo logo and header
    rom.extend_from_slice(&nintendo_logo());
    
    // ROM header
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'-', b'E', b'N', b'C', 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00); // ROM ONLY
    
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0);
    rom.push(0);
    
    // Game code starts at 0x0150
    while rom.len() < 0x0150 { rom.push(0); }
    
    let instructions = game_code();
    for inst in instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    // Z85 alphabet lookup table at 0x8000 (we'll copy to WRAM at 0xC000)
    while rom.len() < 0x8000 { rom.push(0); }
    
    let z85_alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    rom.extend_from_slice(z85_alphabet);
    
    // Pad to minimum ROM size
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

fn game_code() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    
    vec![
        // Initialize
        DI,
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Send test message "Z85\n" to serial output
        // to verify serial communication works
        
        // Send 'Z'
        LD_8_IMMEDIATE(A, b'Z'),
        LD_8_TO_FF_IMMEDIATE(0x01), // Write to SB
        LD_8_IMMEDIATE(A, 0x81),     // Trigger transfer
        LD_8_TO_FF_IMMEDIATE(0x02), // Write to SC
        
        // Send '8'
        LD_8_IMMEDIATE(A, b'8'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Send '5'
        LD_8_IMMEDIATE(A, b'5'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Send newline
        LD_8_IMMEDIATE(A, b'\n'),
        LD_8_TO_FF_IMMEDIATE(0x01),
        LD_8_IMMEDIATE(A, 0x81),
        LD_8_TO_FF_IMMEDIATE(0x02),
        
        // Done - infinite loop
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
