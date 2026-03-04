//! Z85 encoder rewritten with label-based assembler
//! 
//! Compare this to z85-real.rs to see how much cleaner it is!

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

fn main() {
    let rom = build_rom();
    std::fs::write("z85-with-asm.gb", &rom).expect("Failed to write ROM");
    println!("Generated z85-with-asm.gb");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    rom.extend_from_slice(&nintendo_logo());
    rom.extend_from_slice(&[
        b'Z', b'8', b'5', b'A', b'S', b'M', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00);
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); rom.push(0);
    
    // Z85 alphabet at 0x0200
    while rom.len() < 0x0200 { rom.push(0); }
    let z85_alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    rom.extend_from_slice(z85_alphabet);
    
    while rom.len() < 0x0260 { rom.push(0); }
    
    let instructions = game_code();
    for inst in &instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    while rom.len() < 32768 {
        rom.push(0);
    }
    
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
    
    asm.label("COPY_LOOP")
        .inst(LD_8_FROM_SECONDARY(AT_HL_Plus))
        .inst(LD_8_TO_SECONDARY(AT_DE))
        .inst(INC_16(DE))
        .inst(DEC(B))
        .jr_cond(if_NZ, "COPY_LOOP");
    
    // Store test value 0xABCD at 0xC100
    asm.inst(LD_16_IMMEDIATE(HL, 0xC100))
        .inst(LD_8_IMMEDIATE(A, 0xCD))
        .inst(LD_8_INTERNAL(AT_HL, A))
        .inst(INC_16(HL))
        .inst(LD_8_IMMEDIATE(A, 0xAB))
        .inst(LD_8_INTERNAL(AT_HL, A));
    
    // Initialize loop counter
    asm.inst(LD_8_IMMEDIATE(A, 5))
        .inst(LD_16_IMMEDIATE(HL, 0xC105))
        .inst(LD_8_INTERNAL(AT_HL, A));
    
    // === DIGIT LOOP ===
    asm.label("DIGIT_LOOP");
    
    // Load value into DE
    asm.inst(LD_16_IMMEDIATE(HL, 0xC100))
        .inst(LD_8_INTERNAL(E, AT_HL))
        .inst(INC_16(HL))
        .inst(LD_8_INTERNAL(D, AT_HL));
    
    // Divide DE by 85 - this is where the assembler shines!
    asm.inst(LD_16_IMMEDIATE(BC, 0));
    
    asm.label("DIV_LOOP")
        .inst(LD_8_INTERNAL(A, D))
        .inst(OR(A))
        .jr_cond(if_NZ, "DIV_SUBTRACT")
        .inst(LD_8_INTERNAL(A, E))
        .inst(CP_IMMEDIATE(85))
        .jr_cond(if_C, "DIV_EXIT");  // No more manual offset calculation!
    
    asm.label("DIV_SUBTRACT")
        .inst(LD_8_INTERNAL(A, E))
        .inst(SUB_IMMEDIATE(85))
        .inst(LD_8_INTERNAL(E, A))
        .jr_cond(if_NC, "NO_BORROW")  // Clean and obvious!
        .inst(DEC(D));
    
    asm.label("NO_BORROW")
        .inst(INC_16(BC))
        .jr("DIV_LOOP");  // Just "go to DIV_LOOP" - beautiful!
    
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
    
    // Digit storage
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
    
    // Output digits
    asm.inst(LD_16_IMMEDIATE(HL, 0xC110))
        .inst(LD_8_IMMEDIATE(B, 5));
    
    asm.label("OUTPUT_LOOP")
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(INC_16(HL))
        .inst(PUSH(HL))
        .inst(LD_16_IMMEDIATE(HL, 0xC000))
        .inst(ADD(L))
        .inst(LD_8_INTERNAL(L, A))
        .inst(LD_8_IMMEDIATE(A, 0))
        .inst(ADC(H))
        .inst(LD_8_INTERNAL(H, A))
        .inst(LD_8_INTERNAL(A, AT_HL))
        .inst(LD_8_TO_FF_IMMEDIATE(0x01))
        .inst(PUSH_AF)
        .inst(LD_8_IMMEDIATE(A, 0x81))
        .inst(LD_8_TO_FF_IMMEDIATE(0x02))
        .inst(POP_AF)
        .inst(POP(HL))
        .inst(DEC(B))
        .jr_cond(if_NZ, "OUTPUT_LOOP");
    
    // Newline
    asm.inst(LD_8_IMMEDIATE(A, b'\n'))
        .inst(LD_8_TO_FF_IMMEDIATE(0x01))
        .inst(LD_8_IMMEDIATE(A, 0x81))
        .inst(LD_8_TO_FF_IMMEDIATE(0x02));
    
    asm.label("HALT")
        .inst(HALT)
        .jr("HALT");
    
    asm.assemble()
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
