//! Check memory after test-store-trace runs

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-store-trace.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    for _ in 0..100_000 {
        let opex = gb.tick();
        for _ in 0..(opex.t_1 - opex.t_0) {
            gb.video_cycle();
            gb.timer_cycle();
        }
    }
    
    println!("Memory at 0xC100-0xC105:");
    for addr in 0xC100..=0xC105 {
        print!("{:02X} ", gb.read_memory(addr));
    }
    println!();
    
    println!("\nMemory at 0xC110-0xC114:");
    for addr in 0xC110..=0xC114 {
        print!("{:02X} ", gb.read_memory(addr));
    }
    println!();
}
