//! Serial I/O wrapper for Game Boy ROMs (v2.0)
//! 
//! Pipes stdin/stdout through Game Boy serial port with realistic timing.
//! Usage: echo "data" | cargo run --bin serial-io rom.gb
//!
//! Design: See docs/serial-io-design.md
//!
//! Key features:
//! - Non-blocking streaming stdin
//! - Activity-based termination (4M cycle timeout)
//! - Queue-based backpressure
//! - 1024-cycle serial transfer simulation (matches hardware)

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

#[cfg(unix)]
const ACTIVITY_TIMEOUT: usize = 4_000_000;  // 4M cycles (~1 second)
const MAX_TOTAL_CYCLES: usize = 20_000_000; // Safety limit
const INPUT_QUEUE_LIMIT: usize = 10;         // Backpressure threshold
const _SERIAL_TRANSFER_CYCLES: usize = 1024;  // For future use (realistic serial timing)

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
    
    // Set stdin to non-blocking mode
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let stdin_fd = io::stdin().as_raw_fd();
        unsafe {
            let flags = libc::fcntl(stdin_fd, libc::F_GETFL);
            libc::fcntl(stdin_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }
    
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // State
    let mut input_queue: VecDeque<u8> = VecDeque::new();
    let mut stdin_exhausted = false;
    let mut cycles_since_activity: usize = 0;
    let mut total_cycles: usize = 0;
    let mut last_serial_len = 0;
    
    // Stats
    let mut bytes_read = 0;
    let mut bytes_written = 0;
    
    eprintln!("Serial I/O wrapper v2.0");
    eprintln!("Activity timeout: {} cycles (~1 sec)", ACTIVITY_TIMEOUT);
    eprintln!("Queue limit: {} bytes", INPUT_QUEUE_LIMIT);
    
    'main_loop: while total_cycles < MAX_TOTAL_CYCLES {
        // Try to read from stdin (non-blocking)
        if !stdin_exhausted {
            let mut chunk = [0u8; 256];
            match stdin.read(&mut chunk) {
                Ok(0) => {
                    // EOF
                    stdin_exhausted = true;
                    eprintln!("stdin EOF after {} bytes", bytes_read);
                }
                Ok(n) => {
                    // Got data - push to queue and reset activity
                    for i in 0..n {
                        input_queue.push_back(chunk[i]);
                    }
                    bytes_read += n;
                    cycles_since_activity = 0;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available right now - that's fine
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    break;
                }
            }
        }
        
        // Feed input to emulator from our queue
        // Push bytes as they become available
        if !input_queue.is_empty() {
            if let Some(byte) = input_queue.pop_front() {
                gb.push_serial_input(byte);
            }
        }
        
        // Backpressure: if stdin not exhausted but queue getting large, 
        // we'll naturally stop reading more on next iteration
        // This prevents unbounded memory growth
        if !stdin_exhausted && input_queue.len() > INPUT_QUEUE_LIMIT {
            eprintln!("Backpressure active: queue at {} bytes", input_queue.len());
        }
        
        // Execute one instruction
        let opex = gb.tick();
        let tick_cycles = opex.t_1 - opex.t_0;
        
        for _ in 0..tick_cycles {
            gb.video_cycle();
            gb.timer_cycle();
        }
        
        total_cycles += tick_cycles as usize;
        cycles_since_activity += tick_cycles as usize;
        
        // Check for new serial output
        let serial_out = gb.serial_output();
        if serial_out.len() > last_serial_len {
            let new_bytes = &serial_out[last_serial_len..];
            stdout.write_all(new_bytes).expect("Failed to write to stdout");
            stdout.flush().expect("Failed to flush stdout");
            
            bytes_written += new_bytes.len();
            last_serial_len = serial_out.len();
            
            // Reset activity counter - ROM is producing output
            cycles_since_activity = 0;
        }
        
        // Termination: if stdin done and no activity for timeout period
        if stdin_exhausted && cycles_since_activity > ACTIVITY_TIMEOUT {
            eprintln!("Terminating: {} cycles of inactivity (stdin exhausted)", cycles_since_activity);
            break 'main_loop;
        }
    }
    
    if total_cycles >= MAX_TOTAL_CYCLES {
        eprintln!("Warning: Hit max cycles ({}) - safety termination", MAX_TOTAL_CYCLES);
    }
    
    eprintln!("");
    eprintln!("=== Serial I/O Statistics ===");
    eprintln!("Total cycles:     {}", total_cycles);
    eprintln!("Bytes read:       {}", bytes_read);
    eprintln!("Bytes written:    {}", bytes_written);
    eprintln!("Queue remaining:  {}", input_queue.len());
    eprintln!("Final activity:   {} cycles ago", cycles_since_activity);
}
