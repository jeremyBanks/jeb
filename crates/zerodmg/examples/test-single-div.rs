//! Test single division

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-div-only.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    println!("Running test-div-only.gb...");
    
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
    
    println!("\n=== Results at 0xC100 ===");
    let remainder = gb.read_memory(0xC100);
    let quot_low = gb.read_memory(0xC101);
    let quot_high = gb.read_memory(0xC102);
    
    println!("Remainder: {} (expected: 40)", remainder);
    println!("Quotient: {} (expected: 2)", quot_low as u16 | ((quot_high as u16) << 8));
    
    if remainder == 40 && quot_low == 2 && quot_high == 0 {
        println!("\n✅ Division is WORKING correctly!");
    } else {
        println!("\n❌ Division FAILED");
    }
}
