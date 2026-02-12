//! Run a ROM — just capture serial output over time, no instruction trace.

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
    let max_cycles: u64 = 5_000_000_000; // 5B cycles — about 5 seconds of GB time
    let mut last_serial_len = 0;
    let mut serial_text = String::new();

    // Print a status every 500M cycles
    let mut next_status = 500_000_000u64;

    while cycles < max_cycles {
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }
        cycles += tick_cycles;

        // Check for new serial output
        let serial = gb.serial_output();
        if serial.len() > last_serial_len {
            let new_bytes = &serial[last_serial_len..];
            let s = String::from_utf8_lossy(new_bytes);
            // Print each new character with its cycle count
            for c in s.chars() {
                if c == '\n' {
                    eprintln!("[cycle {}] serial: \\n", cycles);
                } else {
                    eprintln!("[cycle {}] serial: '{}'", cycles, c);
                }
            }
            serial_text.push_str(&s);
            last_serial_len = serial.len();
            
            // Check for pass/fail
            if serial_text.contains("Passed") || serial_text.contains("Failed") {
                eprintln!("[cycle {}] --- TEST COMPLETED ---", cycles);
                break;
            }
        }
        
        if cycles >= next_status {
            eprintln!("[cycle {}] --- {} bytes of serial output so far ---", cycles, serial_text.len());
            next_status += 500_000_000;
        }
    }
    
    println!("\n--- After {} cycles ---", cycles);
    println!("Serial output ({} bytes):", serial_text.len());
    println!("{}", serial_text);
}
