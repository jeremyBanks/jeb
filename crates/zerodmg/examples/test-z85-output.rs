//! Quick test to see z85-real.rs serial output

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("z85-real.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    println!("Running z85-real.gb...");
    
    // Run for 500,000 cycles
    let mut cycles = 0;
    while cycles < 500_000 {
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;
        
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }
        
        cycles += tick_cycles;
    }
    
    let output = gb.serial_output();
    
    println!("\n=== Serial Output ===");
    println!("Bytes: {}", output.len());
    println!("String: {:?}", String::from_utf8_lossy(output));
    println!("Hex: {}", output.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
    
    println!("\n=== PC ===");
    println!("0x{:04X}", gb.pc());
    
    println!("\n=== Alphabet at 0xC000 (first 10) ===");
    for i in 0..10 {
        print!("{}", gb.read_memory(0xC000 + i) as char);
    }
    println!();
    
    println!("\n=== Digits at 0xC110 (5 bytes) ===");
    for i in 0..5 {
        print!("{:02X} ", gb.read_memory(0xC110 + i));
    }
    println!();
    
    println!("\n=== Value at 0xC100 (should be quotient after divisions) ===");
    for i in 0..4 {
        print!("{:02X} ", gb.read_memory(0xC100 + i));
    }
    let val_low = gb.read_memory(0xC100);
    let val_high = gb.read_memory(0xC101);
    let value_16 = val_low as u16 | ((val_high as u16) << 8);
    println!(" (value={}, expected: quotient should be 0 after 5 divisions)", value_16);
    
    println!("\n=== Loop counter at 0xC105 ===");
    println!("{:02X} (expected: 00 after 5 iterations)", gb.read_memory(0xC105));
    
    println!("\n=== Memory region 0xC100-0xC120 ===");
    for addr in (0xC100..=0xC120).step_by(16) {
        print!("0x{:04X}: ", addr);
        for i in 0..16 {
            if addr + i <= 0xC120 {
                print!("{:02X} ", gb.read_memory(addr + i));
            }
        }
        println!();
    }
}
