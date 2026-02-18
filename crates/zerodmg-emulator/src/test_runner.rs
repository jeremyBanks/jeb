//! Test runner infrastructure for Blargg test ROMs.

use std::sync::{Arc, Mutex};

use crate::cpu::CPUController;
use crate::memory::MemoryController;
use crate::video::VideoController;
use crate::{GameBoy, Output};

/// Result of running a Blargg test ROM.
pub struct TestResult {
    /// Serial output captured from the test as a string.
    pub output: String,
    /// Number of CPU cycles executed.
    pub cycles: u64,
    /// Final status of the test.
    pub status: TestStatus,
    /// Final PC value (for debugging).
    pub final_pc: u16,
}

/// Status of a test execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// Test is still running (should not be returned).
    Running,
    /// Test passed successfully.
    Passed,
    /// Test failed.
    Failed,
    /// Test timed out (hit cycle limit).
    Timeout,
    /// Test hit an unimplemented feature.
    Unimplemented(String),
}

/// Runner for Blargg test ROMs.
pub struct BlarggTestRunner;

impl BlarggTestRunner {
    /// Run a test ROM for up to max_cycles.
    pub fn run_test(rom: Vec<u8>, max_cycles: u64) -> TestResult {
        let output_buffer = Arc::new(Mutex::new(Output::new()));
        let mut gameboy = GameBoy::new_skip_boot(rom, output_buffer);

        let mut cycles: u64 = 0;
        let mut last_output_len = 0;
        let mut cycles_since_output = 0;
        let mut concluded = false; // true once "Passed"/"Failed" seen in output
        let mut last_pc: u16 = 0xFFFF;
        let mut pc_loop_cycles: u64 = 0; // cycles since last new unique PC seen
        let mut pc_seen_window: std::collections::HashSet<u16> = std::collections::HashSet::new();
        let mut timed_out = false; // true if we exited due to a loop/timeout
        const IDLE_CYCLES_THRESHOLD: u64 = 500_000_000;
        // If no new PC addresses appear for this many cycles and no result found,
        // ROM is stuck in a loop — give up.
        const PC_LOOP_THRESHOLD: u64 = 50_000_000;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            while cycles < max_cycles {
                let opex = gameboy.tick();
                let tick_cycles = opex.t_1 - opex.t_0;

                // Advance video and timer timing
                for _ in 0..tick_cycles {
                    gameboy.video_cycle();
                    gameboy.timer_cycle();
                }

                cycles += tick_cycles;

                // Check for new serial output
                let current_output_len = gameboy.serial_output().len();
                if current_output_len > last_output_len {
                    last_output_len = current_output_len;
                    if !concluded {
                        cycles_since_output = 0;
                    }

                    // Check for test completion markers in output
                    if !concluded {
                        let output = gameboy.serial_output();
                        let output_str = String::from_utf8_lossy(output);
                        if output_str.contains("Passed") || output_str.contains("Failed") {
                            concluded = true;
                        }
                    }
                } else {
                    cycles_since_output += tick_cycles;
                }

                // Exit as soon as test concludes with a trailing newline (serial-output ROMs).
                // `concluded` is set when "Passed"/"Failed" appears anywhere in output.
                // We wait until the last byte is '\n' to ensure the final line is complete.
                if concluded {
                    let output = gameboy.serial_output();
                    if !output.is_empty() && output[output.len() - 1] == b'\n' {
                        break;
                    }
                }

                // Check for ext RAM result (visual-output ROMs that write to 0xA000).
                // The shell initializes final_result to 0x80 ("not yet done").
                // Only exit when it has been updated to a terminal value (0 = pass, 1..N = fail).
                // Check periodically (every ~1M cycles) to avoid overhead.
                if cycles % 1_000_000 < 4 {
                    if read_ext_ram_result(&gameboy).is_some() {
                        break;
                    }
                }

                // Track PC loops: if no new PC addresses appear for a while and there's
                // still no detectable result, the ROM is stuck (e.g. CGB-only ROMs on DMG).
                let current_pc = gameboy.pc();
                if current_pc != last_pc {
                    last_pc = current_pc;
                    if pc_seen_window.insert(current_pc) {
                        // New PC — reset the stall counter and clear the window periodically
                        pc_loop_cycles = 0;
                        if pc_seen_window.len() > 512 {
                            pc_seen_window.clear();
                        }
                    }
                }
                pc_loop_cycles += tick_cycles;
                if pc_loop_cycles > PC_LOOP_THRESHOLD {
                    timed_out = true;
                    break;
                }

                // If no output for a while and we have some output, test might be done
                if cycles_since_output > IDLE_CYCLES_THRESHOLD && last_output_len > 0 {
                    break;
                }
            }
            gameboy
        }));

        match result {
            Ok(gameboy) => {
                let raw = gameboy.serial_output();
                let output = String::from_utf8_lossy(raw).to_string();
                let final_pc = gameboy.pc();

                // Check for result in external RAM — used by visual-output ROMs
                // that don't send serial output but write to 0xA000 on exit.
                // The shell writes a magic marker at 0xA001..=0xA003 ($DE, $B0, $61)
                // and the result byte at 0xA000 (0 = pass, else fail).
                // Text output starts at 0xA004 as a null-terminated string.
                let ext_ram_result = read_ext_ram_result(&gameboy);

                let (status, output) = if let Some((ext_status, ext_text)) = ext_ram_result {
                    (ext_status, ext_text)
                } else if cycles >= max_cycles || timed_out {
                    (TestStatus::Timeout, output)
                } else {
                    (parse_test_status(&output), output)
                };
                TestResult {
                    output,
                    cycles,
                    status,
                    final_pc,
                }
            }
            Err(panic_info) => {
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic".to_string()
                };
                TestResult {
                    output: String::new(),
                    cycles,
                    status: TestStatus::Unimplemented(panic_msg),
                    final_pc: 0,
                }
            }
        }
    }
}

/// Parse the serial output to determine test status.
fn parse_test_status(output: &str) -> TestStatus {
    let output_lower = output.to_lowercase();
    if output_lower.contains("passed") {
        TestStatus::Passed
    } else if output_lower.contains("failed") {
        TestStatus::Failed
    } else {
        TestStatus::Running
    }
}

/// Check for test result written to external RAM by visual-output Blargg ROMs.
///
/// The Blargg shell (shell.s `post_exit`) writes:
/// - 0xA000: final result byte (0 = pass, 1 = general fail, N = "Failed #N")
/// - 0xA001: magic marker $DE
/// - 0xA002: magic marker $B0
/// - 0xA003: magic marker $61
/// - 0xA004..: text output as null-terminated string
///
/// Returns `Some((status, text))` if the magic marker is present.
fn read_ext_ram_result(gameboy: &GameBoy) -> Option<(TestStatus, String)> {
    // Check magic marker
    if gameboy.mem(0xA001) != 0xDE
        || gameboy.mem(0xA002) != 0xB0
        || gameboy.mem(0xA003) != 0x61
    {
        return None;
    }

    let result_byte = gameboy.mem(0xA000);

    // 0x80 is the initial "not yet done" sentinel written by init_text_out.
    // Only treat result as terminal when it has been updated by post_exit.
    if result_byte == 0x80 {
        return None;
    }

    // Read null-terminated text string starting at 0xA004
    let mut text = String::new();
    let mut addr: u16 = 0xA004;
    loop {
        let byte = gameboy.mem(addr);
        if byte == 0 {
            break;
        }
        text.push(byte as char);
        addr = addr.wrapping_add(1);
        if addr == 0xA000 {
            break; // wrap-around guard
        }
    }

    let status = if result_byte == 0 {
        TestStatus::Passed
    } else {
        TestStatus::Failed
    };

    Some((status, text))
}
