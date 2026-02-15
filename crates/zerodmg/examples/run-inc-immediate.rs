//! Run division minimal test and check memory

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-inc-immediate.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    for _ in 0..100_000 {
        let opex = gb.tick();
        for _ in 0..(opex.t_1 - opex.t_0) {
            gb.video_cycle();
            gb.timer_cycle();
        }
    }
    
    println!("=== Memory at 0xC100-0xC102 ===");
    let c1 = gb.read_memory(0xC100);
    let c2 = gb.read_memory(0xC101);
    let c3 = gb.read_memory(0xC102);
    
    println!("After 1st INC: C = {}", c1);
    println!("After 2nd INC: C = {}", c2);
    println!("After 3rd INC: C = {}", c3);
    println!();
    println!("Expected: 1, 2, 3");
    
    if c1 == 1 && c2 == 2 && c3 == 3 {
        println!("\n✅ INC_16(BC) works with SUB/LD operations!");
    } else {
        println!("\n❌ Something wrong");
    }
}
