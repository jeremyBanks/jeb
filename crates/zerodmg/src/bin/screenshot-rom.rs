//! Visual sanity check: run ROM for 10 seconds, screenshot once per second.

use std::env;
use std::fs;
use std::sync::{Arc, Mutex};

use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

// Game Boy clock: ~4.194304 MHz
const CYCLES_PER_SECOND: u64 = 4_194_304;
const TOTAL_SECONDS: u64 = 10;
const SCREENSHOT_INTERVAL: u64 = CYCLES_PER_SECOND; // 1 second
const MAX_CYCLES: u64 = CYCLES_PER_SECOND * TOTAL_SECONDS;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: screenshot-rom <path-to-rom.gb> [output-dir]");
        eprintln!();
        eprintln!("Runs ROM for 10 seconds, takes screenshot every second.");
        eprintln!("Saves frames as frame_001.png, frame_002.png, etc.");
        return;
    }

    let rom_path = &args[1];
    let output_dir = if args.len() >= 3 {
        &args[2]
    } else {
        "screenshots"
    };

    let rom = fs::read(rom_path).expect("Failed to read ROM file");

    println!("=== Screenshot Runner ===");
    println!("ROM: {}", rom_path);
    println!("ROM size: {} bytes", rom.len());
    println!("Output: {}/", output_dir);
    println!("Duration: {} seconds", TOTAL_SECONDS);
    println!();

    // Create output directory
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Initialize emulator
    let output = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, Arc::clone(&output));

    let mut cycle_count = 0u64;
    let mut next_screenshot_cycle = 0u64;
    let mut screenshot_num = 1;

    println!("Running emulator...");

    while cycle_count < MAX_CYCLES {
        // Execute one instruction
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;

        // Advance video and timer for each T-cycle
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }

        cycle_count += tick_cycles;

        if cycle_count >= next_screenshot_cycle {
            // Take screenshot
            let frame = {
                let output_buffer = output.lock().expect("output buffer mutex poisoned");
                output_buffer.display.clone()
            };

            let filename = format!("{}/frame_{:03}.png", output_dir, screenshot_num);
            frame.save(&filename).expect("Failed to save frame");
            
            let elapsed_sec = cycle_count / CYCLES_PER_SECOND;
            println!("  [{}s] Saved {}", elapsed_sec, filename);

            screenshot_num += 1;
            next_screenshot_cycle += SCREENSHOT_INTERVAL;
        }
    }

    println!();
    println!("Done! Captured {} frames in {}/", screenshot_num - 1, output_dir);
}
