//! Test Z85 encoder serial output

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};
use std::fs;

fn main() {
    let rom_path = std::env::args().nth(1).unwrap_or_else(|| "z85-encoder.gb".to_string());
    let rom = fs::read(&rom_path).expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    println!("Running {}...", rom_path);
    
    // Run for 1,000,000 cycles
    let mut cycles = 0;
    while cycles < 1_000_000 {
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;
        
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }
        
        cycles += tick_cycles;
    }
    
    let output = gb.serial_output();
    
    // Check PC
    println!("\n=== CPU State ===");
    println!("PC: 0x{:04X}", gb.pc());
    
    println!("\n=== Serial Output ===");
    println!("Bytes: {}", output.len());
    println!("Hex: {:02X?}", output);
    println!("ASCII: {:?}", String::from_utf8_lossy(output));
    
    // Check if alphabet was copied to RAM
    println!("\n=== Z85 Alphabet at 0xC000 ===");
    print!("First 10 bytes: ");
    for i in 0..10 {
        let byte = gb.read_memory(0xC000 + i);
        print!("{:02X} ", byte);
    }
    println!();
    print!("As chars: ");
    for i in 0..10 {
        let byte = gb.read_memory(0xC000 + i);
        print!("{}", if byte >= 32 && byte < 127 { byte as char } else { '.' });
    }
    println!();
}
