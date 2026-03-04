//! Run division trace ROM

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;
use std::sync::{Arc, Mutex};

fn main() {
    let rom = std::fs::read("test-div-trace.gb").expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    
    println!("Running test-div-trace.gb...");
    
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
    println!("Expected: \"1246 8\\n\" or \"124 8\\n\"");
    
    println!("\n=== Results at 0xC100 ===");
    let e_after = gb.read_memory(0xC100);
    let c = gb.read_memory(0xC101);
    let b = gb.read_memory(0xC102);
    
    println!("E after subtraction: {} (expected: 125)", e_after);
    println!("BC (quotient): {} (expected: 1)", c as u16 | ((b as u16) << 8));
    
    println!("\nTrace interpretation:");
    println!("1 = started");
    println!("2 = D == 0 (correct, D is 0)");
    println!("4 = E >= 85 (correct, 210 >= 85)");
    println!("6 = no borrow from subtraction (correct, 210-85=125)");
    println!("8 = incremented BC");
}
