//! Run a ROM with instruction tracing to debug test failures.

use std::env;
use std::fs;
use std::sync::{Arc, Mutex};

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: trace-rom <path-to-rom.gb>");
        return;
    }

    let rom = fs::read(&args[1]).expect("Failed to read ROM");
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);

    let mut cycles: u64 = 0;
    let max_instrs = 100_000u64;
    let mut last_instrs: Vec<String> = Vec::new();
    let keep_last = 50;

    for i in 0..max_instrs {
        let _pc = gb.pc();
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }
        cycles += tick_cycles;
        
        let mut line = format!("{:6}: {:<24}", format!("{}", opex.source), format!("{}", opex.instruction));
        if let Some(ref tracer) = opex.tracer {
            line.push_str(&format!(" ; {}", tracer()));
        }
        
        if i < 30 {
            println!("{}", line);
        }
        
        last_instrs.push(line);
        if last_instrs.len() > keep_last {
            last_instrs.remove(0);
        }
    }
    
    println!("\n--- Last {} instructions (of {} total, {} cycles) ---", keep_last, max_instrs, cycles);
    for line in &last_instrs {
        println!("{}", line);
    }
    
    println!("\n--- After {} cycles ---", cycles);
    println!("Final PC: 0x{:04X}", gb.pc());
    println!("Serial output: {} bytes", gb.serial_output().len());
    if !gb.serial_output().is_empty() {
        println!("Output: {}", String::from_utf8_lossy(gb.serial_output()));
    }
}
