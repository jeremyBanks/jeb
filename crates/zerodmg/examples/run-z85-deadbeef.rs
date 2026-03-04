//! Run division minimal test and check memory

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-z85-deadbeef.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    for _ in 0..500_000 {  // More cycles for full 32-bit encoding
        let opex = gb.tick();
        for _ in 0..(opex.t_1 - opex.t_0) {
            gb.video_cycle();
            gb.timer_cycle();
        }
    }
    
    let output = gb.serial_output();
    
    println!("=== Serial Output ===");
    println!("String: {:?}", String::from_utf8_lossy(output));
    println!();
    println!("Expected: \"006+4\\n\"");
    println!("Test value: 0xBEEF (48879 decimal)");
    
    if output == b"006+4\n" {
        println!("\n✅ Z85 encoding WORKS for 0xBEEF!");
    } else {
        println!("\n❌ Output mismatch");
    }
}
