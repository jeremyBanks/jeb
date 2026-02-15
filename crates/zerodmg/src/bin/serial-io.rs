//! Serial I/O wrapper for Game Boy ROMs
//! 
//! Pipes stdin/stdout byte-by-byte through the Game Boy serial port.
//! Usage: echo "data" | cargo run --bin serial-io rom.gb
//! 
//! The wrapper doesn't know about the binary format - just passes bytes through.

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rom.gb>", args[0]);
        std::process::exit(1);
    }
    
    let rom_path = &args[1];
    let rom = fs::read(rom_path).expect("Failed to read ROM");
    
    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    
    let mut input_buffer = Vec::new();
    let mut input_pos = 0;
    
    // Read all stdin into buffer
    let stdin_bytes = stdin.read_to_end(&mut input_buffer).expect("Failed to read stdin");
    
    let mut last_serial_len = 0;
    let mut cycles: usize = 0;
    const MAX_CYCLES: usize = 10_000_000;
    const FEED_RATE: usize = 1000; // Feed one input byte every N cycles
    
    eprintln!("Running ROM with {} input bytes", stdin_bytes);
    
    while cycles < MAX_CYCLES {
        // Feed input bytes gradually
        if cycles % FEED_RATE == 0 && input_pos < input_buffer.len() {
            gb.push_serial_input(input_buffer[input_pos]);
            input_pos += 1;
        }
        
        // Execute one instruction
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;
        
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }
        
        cycles += tick_cycles as usize;
        
        // Check for new serial output
        let serial_out = gb.serial_output();
        if serial_out.len() > last_serial_len {
            let new_bytes = &serial_out[last_serial_len..];
            stdout.write_all(new_bytes).expect("Failed to write to stdout");
            stdout.flush().expect("Failed to flush stdout");
            last_serial_len = serial_out.len();
        }
        
        // Stop if all input consumed and no new output for a while
        if input_pos >= input_buffer.len() && cycles > 100_000 {
            if gb.serial_output().len() == last_serial_len {
                // No new output in last cycle - assume done
                break;
            }
        }
    }
    
    if cycles >= MAX_CYCLES {
        eprintln!("Warning: Hit max cycles ({}) before ROM finished", MAX_CYCLES);
    }
    
    eprintln!("Completed in {} cycles, consumed {} input bytes, produced {} output bytes",
        cycles, input_pos, gb.serial_output().len());
}
