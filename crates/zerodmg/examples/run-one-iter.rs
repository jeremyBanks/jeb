//! Run one iteration test

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-one-iter.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    println!("Running test-one-iter.gb...");
    
    // Run for 100,000 cycles
    let mut cycles = 0;
    while cycles < 100_000 {
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
    println!("String: {:?}", String::from_utf8_lossy(output));
    println!("Expected: \"123\\n\"");
    
    println!("\n=== Remainder at 0xC110 ===");
    let remainder = gb.read_memory(0xC110);
    println!("{} (expected: 40)", remainder);
    
    println!("\n=== Quotient at 0xC100 (after store) ===");
    let quot_low = gb.read_memory(0xC100);
    let quot_high = gb.read_memory(0xC101);
    let quotient = quot_low as u16 | ((quot_high as u16) << 8);
    println!("{} (expected: 2)", quotient);
    
    if remainder == 40 && quotient == 2 {
        println!("\n✅ One iteration WORKING!");
    } else {
        println!("\n❌ One iteration FAILED");
    }
}
