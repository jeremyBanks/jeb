//! Debug tool: instrument Blargg ROM execution with timer state tracking.
//! Usage: debug-blargg <test> [--tima-trace] [--max N]

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use zerodmg_codes::roms::blargg_tests;
use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

fn get_rom(name: &str) -> Option<Vec<u8>> {
    match name {
        "cpu_instrs"    => Some(blargg_tests::cpu_instrs().to_bytes()),
        "halt_bug"      => Some(blargg_tests::halt_bug().to_bytes()),
        "instr_timing"  => Some(blargg_tests::instr_timing().to_bytes()),
        "mem_timing"    => Some(blargg_tests::mem_timing().to_bytes()),
        "mem_timing_2"  => Some(blargg_tests::mem_timing_2().to_bytes()),
        "interrupt_time"=> Some(blargg_tests::interrupt_time().to_bytes()),
        "dmg_sound"     => Some(blargg_tests::dmg_sound().to_bytes()),
        "oam_bug"       => Some(blargg_tests::oam_bug().to_bytes()),
        "cgb_sound"     => Some(blargg_tests::cgb_sound().to_bytes()),
        _               => std::fs::read(name).ok(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: debug-blargg <test> [--tima-trace] [--max N]");
        return;
    }
    let test_name = &args[1];
    let tima_trace = args.contains(&"--tima-trace".to_string());
    let max_instructions: u64 = args.windows(2)
        .find(|w| w[0] == "--max")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(10_000_000);

    let rom = match get_rom(test_name) {
        Some(r) => r,
        None => { eprintln!("Unknown test: {}", test_name); return; }
    };

    let output_buffer = Arc::new(Mutex::new(Output::new()));
    let mut gb = GameBoy::new_skip_boot(rom, output_buffer);

    let mut instr_count: u64 = 0;
    let mut seen_pcs: HashSet<u16> = HashSet::new();
    let mut last_serial_len = 0;

    // TIMA trace: track writes/reads to 0xFF05
    let mut tima_write_count: u64 = 0;
    let mut tima_read_values: Vec<u8> = Vec::new(); // values seen on read after write
    let mut last_was_tima_write = false;

    loop {
        let pc_before = gb.pc();
        let instr_debug = format!("{:?}", {
            // peek at instruction without executing — just look at the debug string after tick
            ""
        });
        let _ = instr_debug;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gb.tick()
        }));

        let opex = match result {
            Ok(o) => o,
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() { s.clone() }
                    else if let Some(s) = e.downcast_ref::<&str>() { s.to_string() }
                    else { "unknown panic".to_string() };
                println!("[{:8}] PC=0x{:04X}  PANIC: {}", instr_count, pc_before, msg);
                break;
            }
        };

        let tick_cycles = opex.t_1 - opex.t_0;
        let tima_before = gb.read_memory(0xFF05);

        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }

        let tima_after = gb.read_memory(0xFF05);
        let div_after = gb.read_memory(0xFF04);
        let tac = gb.read_memory(0xFF07);

        if tima_trace {
            let instr_str = format!("{:?}", opex.instruction);
            // Detect TIMA write: LDH_TO_FF with offset 5, or LD to memory 0xFF05
            let is_tima_write = instr_str.contains("LD_8_TO_FF_IMMEDIATE(5)")
                || instr_str.contains("55301"); // 0xD805? no - 0xFF05 = 65285
            let is_tima_read = instr_str.contains("LD_8_FROM_FF_IMMEDIATE(5)")
                || (instr_str.contains("LD_8_FROM_MEMORY") && instr_str.contains("65285"));

            if is_tima_write {
                tima_write_count += 1;
                last_was_tima_write = true;
                if tima_trace {
                    // Read raw div_counter via div register * 256 (approx) + read memory
                    // We can get raw div by reading the internal state
                    let raw_div = gb.raw_div_counter();
                    println!("[{:8}] TIMA_WRITE  PC=0x{:04X}  before={} after={}  div_raw={} div_low={}  div%16={}  tac={}",
                        instr_count, pc_before, tima_before, tima_after,
                        raw_div, raw_div & 0xFF, raw_div % 16, tac);
                }
            } else if is_tima_read && last_was_tima_write {
                last_was_tima_write = false;
                tima_read_values.push(tima_after);
                if tima_trace {
                    let raw_div = gb.raw_div_counter();
                    println!("[{:8}] TIMA_READ   PC=0x{:04X}  value={}  div_raw={}  (write #{})",
                        instr_count, pc_before, tima_after, raw_div, tima_write_count);
                }
            } else {
                last_was_tima_write = false;
            }
        }

        // Print new unique PCs
        let is_new = seen_pcs.insert(pc_before);
        if is_new && !tima_trace {
            let sp = gb.cpu_state().8;
            println!("[{:8}] PC=0x{:04X}  {:?}  SP=0x{:04X}",
                instr_count, pc_before, opex.instruction, sp);
        }

        // Serial output
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
            if tima_trace {
                println!("TIMA writes: {}", tima_write_count);
                println!("TIMA reads after write (first 20): {:?}",
                    &tima_read_values[..tima_read_values.len().min(20)]);
                let zeros = tima_read_values.iter().filter(|&&v| v == 0).count();
                println!("Reads where TIMA=0: {} / {}", zeros, tima_read_values.len());
            }
            break;
        }
    }
}
