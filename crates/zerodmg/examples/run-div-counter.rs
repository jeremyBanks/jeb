//! Run division minimal test and check memory

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-div-counter.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    for _ in 0..100_000 {
        let opex = gb.tick();
        for _ in 0..(opex.t_1 - opex.t_0) {
            gb.video_cycle();
            gb.timer_cycle();
        }
    }
    
    println!("=== Memory at 0xC100-0xC104 ===");
    let c = gb.read_memory(0xC100);
    let b = gb.read_memory(0xC101);
    let remainder = gb.read_memory(0xC102);
    let counter = gb.read_memory(0xC104);
    
    println!("C (quotient low): {}", c);
    println!("B (quotient high): {}", b);
    println!("Quotient (BC): {}", c as u16 | ((b as u16) << 8));
    println!("Remainder: {}", remainder);
    println!("Loop iterations: {}", counter);
    println!();
    println!("Expected for 210÷85: quotient=2, remainder=40, iterations=2");
    
    if counter == 2 {
        println!("\n✅ Loop executed {} times (correct!)", counter);
        if c != 2 {
            println!("❌ But BC is {} instead of 2 - INC_16 not working in loop!", c);
        }
    } else {
        println!("\n❌ Loop executed {} times (should be 2)", counter);
    }
}
