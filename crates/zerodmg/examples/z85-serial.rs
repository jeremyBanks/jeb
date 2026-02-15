//! Z85 encoder with serial I/O
//! 
//! Reads 4 bytes from serial input, encodes to 5 Z85 chars, outputs to serial
//! Usage: echo -ne '\xAB\xCD\x12\x34' | cargo run --bin serial-io z85-serial.gb

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-serial.gb", &rom).expect("Failed");
    println!("Generated z85-serial.gb");
    println!("Usage: echo -ne '\\xAB\\xCD\\x12\\x34' | cargo run --bin serial-io z85-serial.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&[
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
        0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
        0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
        0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
        0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
        0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    ]);
    rom.extend_from_slice(&[b'Z', b'8', b'5', b'I', b'O', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    // Z85 alphabet at 0x0200
    while rom.len() < 0x0200 { rom.push(0); }
    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    rom.extend_from_slice(alphabet);
    
    while rom.len() < 0x0260 { rom.push(0); }
    
    for inst in &game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 { rom.push(0); }
    rom
}

fn game_code() -> Vec<Instruction> {
    let mut asm = Assembler::new();
    
    asm.inst(DI)
        .inst(LD_16_IMMEDIATE(SP, 0xFFFE));
    
    // Copy alphabet to RAM
    asm.inst(LD_16_IMMEDIATE(HL, 0x0200))
        .inst(LD_16_IMMEDIATE(DE, 0xC000))
        .inst(LD_8_IMMEDIATE(B, 85));
    
    asm.label("COPY_ALPHA")
        .inst(LD_8_FROM_SECONDARY(AT_HL_Plus))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC(B))
        .jr_cond(if_NZ, "COPY_ALPHA");
    
    // === MAIN LOOP ===
    asm.label("LOOP");
    
    // Read 4 bytes from serial input (stored at 0xC100-0xC103)
    for i in 0..4 {
        asm.inst(LD_8_IMMEDIATE(A, 0x80))  // Request input
            .inst(LD_8_TO_FF_IMMEDIATE(0x02))
            .inst(LD_8_FROM_FF_IMMEDIATE(0x01))  // Read byte
            .inst(LD_16_IMMEDIATE(HL, 0xC100 + i))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }
    
    // Initialize digit loop counter (5 iterations)
    asm.inst(LD_8_IMMEDIATE(A, 5))
        .inst(LD_16_IMMEDIATE(HL, 0xC105))
        .inst(LD_8_INTERNAL(AT_HL, A));
    
    // === DIGIT LOOP ===
    asm.label("DIGIT_LOOP");
    
    // Load 32-bit value from 0xC100 into DE (low 16 bits)
    asm.inst(LD_16_IMMEDIATE(HL, 0xC100))
        .inst(LD_8_INTERNAL(E, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_INTERNAL(D, AT_HL));
    
    // Divide DE by 85
    asm.inst(LD_16_IMMEDIATE(BC, 0));
    
    asm.label("DIV_LOOP")
        .inst(LD_8_INTERNAL(A, D))
        .inst(OR(A))
        .jr_cond(if_NZ, "DIV_SUB")
        .inst(LD_8_INTERNAL(A, E))
        .inst(CP_IMMEDIATE(85))
        .jr_cond(if_C, "DIV_EXIT");
    
    asm.label("DIV_SUB")
        .inst(LD_8_INTERNAL(A, E))
        .inst(SUB_IMMEDIATE(85))
        .inst(LD_8_INTERNAL(E, A))
        .jr_cond(if_NC, "NO_BORROW")
        .inst(DEC(D));
    
    asm.label("NO_BORROW")
        .inst(INC_16(BC))
        .jr("DIV_LOOP");
    
    asm.label("DIV_EXIT")
        .inst(LD_8_INTERNAL(A, E));
    
    // Save remainder
    asm.inst(PUSH_AF);
    
    // Store quotient back
    asm.inst(LD_16_IMMEDIATE(HL, 0xC100))
        .inst(LD_8_INTERNAL(A, C))
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(INC_16(HL))
        .inst(LD_8_INTERNAL(A, B))
        .inst(LD_8_INTERNAL(AT_HL, A));
    
    // Get loop counter, calculate digit address
    asm.inst(LD_16_IMMEDIATE(HL, 0xC105))
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(LD_16_IMMEDIATE(HL, 0xC106))
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(LD_16_IMMEDIATE(HL, 0xC10F))
        .inst(ADD(L))
        .inst(LD_8_INTERNAL(L, A))
        .inst(POP_AF)
        .inst(LD_8_INTERNAL(AT_HL, A));
    
    // Decrement counter
    asm.inst(LD_16_IMMEDIATE(HL, 0xC106))
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(DEC(A))
        .inst(LD_16_IMMEDIATE(HL, 0xC105))
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(OR(A))
        .jr_cond(if_NZ, "DIGIT_LOOP");
    
    // === OUTPUT DIGITS ===
    asm.inst(LD_16_IMMEDIATE(HL, 0xC110))
        .inst(LD_8_IMMEDIATE(B, 5));
    
    asm.label("OUTPUT_LOOP")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(PUSH(HL));
    
    // Alphabet lookup
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(ADD(L))
        .inst(LD_8_INTERNAL(L, A))
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(ADC(H))
        .inst(LD_8_INTERNAL(H, A))
        .inst(LD_8_INTERNAL(A, AT_HL));
    
    // Output to serial
    asm.inst(LD_8_TO_FF_IMMEDIATE(0x01))
        .inst(PUSH_AF)
        .inst(LD_8_IMMEDIATE(A, 0x81))
        .inst(LD_8_TO_FF_IMMEDIATE(0x02))
        .inst(POP_AF);
    
    asm.inst(POP(HL))
        .inst(DEC(B))
        .jr_cond(if_NZ, "OUTPUT_LOOP");
    
    // Loop back for next 4 bytes (use JP for long jumps)
    asm.jp("LOOP");
    
    asm.assemble()
}
