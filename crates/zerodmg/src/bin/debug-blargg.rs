//! Debug tool: find EI instruction and interrupt handler setup in Blargg ROMs.

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use zerodmg_codes::roms::blargg_tests;
use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

fn get_rom(name: &str) -> Option<Vec<u8>> {
    match name {
        "cpu_instrs" => Some(blargg_tests::cpu_instrs().to_bytes()),
        "halt_bug" => Some(blargg_tests::halt_bug().to_bytes()),
        "instr_timing" => Some(blargg_tests::instr_timing().to_bytes()),
        "mem_timing" => Some(blargg_tests::mem_timing().to_bytes()),
        "mem_timing_2" => Some(blargg_tests::mem_timing_2().to_bytes()),
        "interrupt_time" => Some(blargg_tests::interrupt_time().to_bytes()),
        "dmg_sound" => Some(blargg_tests::dmg_sound().to_bytes()),
        "oam_bug" => Some(blargg_tests::oam_bug().to_bytes()),
        "cgb_sound" => Some(blargg_tests::cgb_sound().to_bytes()),
        _ => std::fs::read(name).ok(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: debug-blargg <test-name>");
        return;
    }

    let test_name = &args[1];
    let rom = match get_rom(test_name) {
        Some(r) => r,
        None => { eprintln!("Unknown test: {}", test_name); return; }
    };

    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);

    let mut instr_count: u64 = 0;
    let mut seen_pcs: HashSet<u16> = HashSet::new();
    let max_instructions: u64 = 10_000_000;
    let mut last_serial_len = 0;

    loop {
        let pc_before = gb.pc();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gb.tick()
        }));

        let opex = match result {
            Ok(o) => o,
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() { s.clone() }
                    else if let Some(s) = e.downcast_ref::<&str>() { s.to_string() }
                    else { "unknown panic".to_string() };
                println!("[{:7}] PC=0x{:04X}  PANIC: {}", instr_count, pc_before, msg);
                break;
            }
        };

        let tick_cycles = opex.t_1 - opex.t_0;
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }

        let is_new = seen_pcs.insert(pc_before);

        // Print new unique PCs
        if is_new {
            let sp = gb.cpu_state().8;
            println!("[{:7}] PC=0x{:04X}  {:?}  SP=0x{:04X}",
                instr_count, pc_before, opex.instruction, sp);
        }

        // Print serial output
        let serial = gb.serial_output();
        if serial.len() > last_serial_len {
            let new_text = String::from_utf8_lossy(&serial[last_serial_len..]);
            println!("<<SERIAL @{}: {}>>", instr_count, new_text);
            last_serial_len = serial.len();
        }

        instr_count += 1;
        if instr_count >= max_instructions {
            let sp = gb.cpu_state().8;
            println!("--- stopped after {} instructions. PC=0x{:04X} SP=0x{:04X} ---",
                max_instructions, gb.pc(), sp);
            break;
        }
    }
}
