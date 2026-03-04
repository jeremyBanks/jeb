//! Run division direct test

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-bc-before-div.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    println!("Running test-bc-before-div.gb...");
    
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
    println!("Bytes: {:?}", output);
    
    if output.len() >= 3 {
        let remainder_char = output[0];
        let quot_low_char = output[1];
        let quot_high_char = output[2];
        
        println!("\nRemainder: {} - 48 = {}", remainder_char as char, remainder_char.saturating_sub(48));
        println!("Quotient low: {} - 48 = {}", quot_low_char as char, quot_low_char.saturating_sub(48));
        println!("Quotient high: {} - 48 = {}", quot_high_char as char, quot_high_char.saturating_sub(48));
        
        println!("\nExpected: remainder=40, quotient=2");
        
        let rem_val = remainder_char.saturating_sub(48);
        let quot_val = quot_low_char.saturating_sub(48);
        
        if rem_val == 40 && quot_val == 2 {
            println!("\n✅ Division is WORKING!");
        }
    }
}
