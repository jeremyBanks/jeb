//! Debug: Test Conway neighbor counting via serial output
//! 
//! Places the glider, then for cell 146 (7,6) which should have 3 live neighbors:
//! - Counts neighbors
//! - Outputs count via serial (should be '3')
//! - Also outputs current cell state ('1' = alive)
//! Then outputs expected next state ('1' = survive)
//!
//! Serial output format: "N=<count>,S=<state>,NX=<next>\n" repeated each generation

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

const W: usize = 20;
const H: usize = 18;
const CELLS: usize = W * H;
const GRID_CUR: u16 = 0xC000;

fn main() {
    let rom = build_rom();
    std::fs::write("conway-debug.gb", &rom).expect("Failed");
    println!("Generated conway-debug.gb");
    println!("Run with: cargo run -p zerodmg --bin serial-rom -- conway-debug.gb 2>/dev/null");
    println!("Expected output: 'C=3 S=1 NX=1' (cell 146 has 3 neighbors, alive, survives)");
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
    rom.extend_from_slice(&[b'D', b'E', b'B', b'U', b'G', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    while rom.len() < 0x0150 { rom.push(0); }
    for inst in &game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    while rom.len() < 32768 { rom.push(0); }
    rom
}

fn neighbor_addr(y: usize, x: usize, dy: i32, dx: i32) -> u16 {
    let ny = ((y as i32 + dy).rem_euclid(H as i32)) as usize;
    let nx = ((x as i32 + dx).rem_euclid(W as i32)) as usize;
    GRID_CUR + (ny * W + nx) as u16
}

/// Serial output: LD A, char; LD (0xFF01), A; LD A, 0x81; LD (0xFF02), A
fn serial_char(asm: &mut Assembler, c: u8) {
    asm.inst(LD_8_IMMEDIATE(A, c))
        .inst(LD_8_TO_FF_IMMEDIATE(0x01))  // Serial data register
        .inst(LD_8_IMMEDIATE(A, 0x81))      // Start transfer (internal clock)
        .inst(LD_8_TO_FF_IMMEDIATE(0x02));  // Serial control
}

/// Output A as decimal digit + '0'
fn serial_digit(asm: &mut Assembler) {
    // A = digit value (0-9), output A + '0'
    asm.inst(ADD_IMMEDIATE(b'0'))
        .inst(LD_8_TO_FF_IMMEDIATE(0x01))
        .inst(LD_8_IMMEDIATE(A, 0x81))
        .inst(LD_8_TO_FF_IMMEDIATE(0x02));
}

fn game_code() -> Vec<Instruction> {
    let mut asm = Assembler::new();

    asm.inst(DI)
        .inst(LD_16_IMMEDIATE(SP, 0xFFFE));

    // Clear grid
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR))
        .inst(LD_16_IMMEDIATE(BC, CELLS as u16))
        .inst(LD_8_IMMEDIATE(A, 0));
    asm.label("CLR")
        .inst(LD_8_TO_SECONDARY(AT_HL_Plus))
        .inst(DEC_16(BC))
        .inst(LD_8_INTERNAL(A, B))
        .inst(OR(C))
        .jr_cond(if_NZ, "CLR");

    // Place glider: (5,6), (6,7), (7,5), (7,6), (7,7)
    for (y, x) in [(5usize, 6usize), (6, 7), (7, 5), (7, 6), (7, 7)] {
        let addr = GRID_CUR + (y * W + x) as u16;
        asm.inst(LD_16_IMMEDIATE(HL, addr))
            .inst(LD_8_IMMEDIATE(A, 1))
            .inst(LD_8_INTERNAL(AT_HL, A));
    }

    // Test: count neighbors of cell 146 (y=7, x=6)
    // Expected neighbors: (6,7)=alive, (7,5)=alive, (7,7)=alive → count=3
    let test_cell = 146usize;
    let test_y = test_cell / W;
    let test_x = test_cell % W;

    let neighbor_deltas: [(i32, i32); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        ( 0, -1),           ( 0, 1),
        ( 1, -1), ( 1, 0), ( 1, 1),
    ];

    let neighbors: Vec<u16> = neighbor_deltas.iter()
        .map(|&(dy, dx)| neighbor_addr(test_y, test_x, dy, dx))
        .collect();

    // Count neighbors
    asm.inst(LD_8_FROM_MEMORY_IMMEDIATE(neighbors[0]));
    for &addr in &neighbors[1..] {
        asm.inst(LD_16_IMMEDIATE(HL, addr));
        asm.inst(ADD(AT_HL));
    }
    // A = neighbor count — save it
    asm.inst(LD_8_INTERNAL(B, A));

    // Output "C=" then count digit
    serial_char(&mut asm, b'C');
    serial_char(&mut asm, b'=');
    asm.inst(LD_8_INTERNAL(A, B));
    serial_digit(&mut asm);
    serial_char(&mut asm, b' ');

    // Load current state of cell 146
    asm.inst(LD_16_IMMEDIATE(HL, GRID_CUR + test_cell as u16));
    asm.inst(LD_8_INTERNAL(C, AT_HL)); // C = current state

    // Output "S=" then state digit
    serial_char(&mut asm, b'S');
    serial_char(&mut asm, b'=');
    asm.inst(LD_8_INTERNAL(A, C));
    serial_digit(&mut asm);
    serial_char(&mut asm, b' ');

    // Compute next state
    asm.inst(LD_8_INTERNAL(A, B)); // A = count
    asm.inst(CP_IMMEDIATE(3));
    asm.jr_cond(if_Z, "WRITE1");
    asm.inst(CP_IMMEDIATE(2));
    asm.jr_cond(if_NZ, "WRITE0");
    asm.inst(LD_8_INTERNAL(A, C));
    asm.inst(OR(A));
    asm.jr_cond(if_NZ, "WRITE1");

    asm.label("WRITE0");
    asm.inst(LD_8_IMMEDIATE(C, 0));
    asm.jr("WRITEDONE");
    asm.label("WRITE1");
    asm.inst(LD_8_IMMEDIATE(C, 1));
    asm.label("WRITEDONE");

    // Output "NX=" then next state digit
    serial_char(&mut asm, b'N');
    serial_char(&mut asm, b'X');
    serial_char(&mut asm, b'=');
    asm.inst(LD_8_INTERNAL(A, C));
    serial_digit(&mut asm);
    serial_char(&mut asm, b'\n');

    // Halt
    asm.inst(HALT)
        .inst(JR(-1));

    asm.assemble()
}
