//! Run any Game Boy ROM file through the test runner.

use std::env;
use std::fs;

use zerodmg_emulator::test_runner::{BlarggTestRunner, TestStatus};

const MAX_CYCLES: u64 = 2_000_000_000; // 2 billion cycles

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: run-rom <path-to-rom.gb>");
        return;
    }

    let rom_path = &args[1];
    let rom = fs::read(rom_path).expect("Failed to read ROM file");

    println!("=== Running ROM: {} ===", rom_path);
    println!("ROM size: {} bytes", rom.len());
    println!();

    let result = BlarggTestRunner::run_test(rom, MAX_CYCLES);

    println!("--- Serial Output ---");
    if result.output.is_empty() {
        println!("(no output)");
    } else {
        println!("{}", result.output);
        // Also show hex
        print!("Hex: ");
        for b in result.output.bytes() {
            print!("{:02X} ", b);
        }
        println!();
    }
    println!();

    println!("--- Result ---");
    println!("Cycles: {}", result.cycles);
    println!("Final PC: 0x{:04X}", result.final_pc);
    print!("Status: ");
    match &result.status {
        TestStatus::Running => println!("RUNNING (idle)"),
        TestStatus::Passed => println!("✓ PASSED"),
        TestStatus::Failed => println!("✗ FAILED"),
        TestStatus::Timeout => println!("⏱ TIMEOUT"),
        TestStatus::Unimplemented(msg) => println!("⚠ {}", msg),
    }
}
