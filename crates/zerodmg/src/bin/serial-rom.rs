//! Run a Game Boy ROM with serial I/O piped to stdin/stdout.
//! 
//! Usage: serial-rom <rom.gb> [--sync]
//! 
//! --sync: Run at real Game Boy speed (~4.19 MHz)
//! Without --sync: Run at unlimited speed

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

const CYCLES_PER_SECOND: u64 = 4_194_304;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: serial-rom <rom.gb> [--sync]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --sync    Run at real Game Boy speed (~4.19 MHz)");
        eprintln!("            Without this, runs at unlimited speed");
        return;
    }

    let rom_path = &args[1];
    let sync_clock = args.contains(&"--sync".to_string());

    let rom = fs::read(rom_path).expect("Failed to read ROM file");

    eprintln!("=== Serial ROM Runner ===");
    eprintln!("ROM: {}", rom_path);
    eprintln!("Clock: {}", if sync_clock { "synced" } else { "unlimited" });
    eprintln!();

    // Initialize emulator
    let output = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, Arc::clone(&output));

    let mut cycle_count = 0u64;
    let start_time = Instant::now();
    let mut last_sync_time = start_time;

    // Read stdin in non-blocking mode (would need a separate thread for real impl)
    // For now, just run the ROM and collect serial output
    
    let mut serial_output = Vec::new();
    
    loop {
        // Execute one instruction
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;

        // Advance video and timer
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }

        cycle_count += tick_cycles;

        // Check for new serial output
        let new_output = gb.serial_output();
        if new_output.len() > serial_output.len() {
            let new_bytes = &new_output[serial_output.len()..];
            io::stdout().write_all(new_bytes).ok();
            io::stdout().flush().ok();
            serial_output.extend_from_slice(new_bytes);
        }

        // Clock sync (if enabled)
        if sync_clock && cycle_count % 10000 == 0 {
            let elapsed = start_time.elapsed();
            let expected_elapsed = Duration::from_secs_f64(cycle_count as f64 / CYCLES_PER_SECOND as f64);
            
            if expected_elapsed > elapsed {
                let sleep_time = expected_elapsed - elapsed;
                std::thread::sleep(sleep_time);
            }
        }

        // Exit condition: no output for a while and PC is looping
        // (This is a simple heuristic - real impl would be more sophisticated)
        if cycle_count > 100_000_000 {
            break;
        }
    }

    eprintln!();
    eprintln!("=== Execution Complete ===");
    eprintln!("Cycles: {}", cycle_count);
    eprintln!("Elapsed: {:.2}s", start_time.elapsed().as_secs_f64());
}
