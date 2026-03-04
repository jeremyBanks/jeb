//! ROM Runner - Unified script-based Game Boy ROM execution
//!
//! Supports:
//! - TAS-style input sequences
//! - Screenshot capture
//! - Serial I/O testing
//! - Deterministic, automatable execution
//!
//! See docs/rom-runner-design.md for full specification.

use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use zerodmg_emulator::{GameBoy, Output};
use zerodmg_emulator::cpu::CPUController;
use zerodmg_emulator::video::VideoController;

// Constants
const DEFAULT_TIMEOUT: usize = 20_000_000;  // 20M cycles
const CYCLES_PER_FRAME: usize = 70_224;     // ~60Hz

#[derive(Debug, Clone)]
enum Command {
    Wait { cycles: usize },
    WaitFrames { frames: usize },
    Halt,
    Timeout { cycles: usize },
    Log { message: String },
    
    // Input control
    Input { buttons: Vec<Button> },
    Release { buttons: Vec<Button> },
    ReleaseAll,
    
    // Serial I/O
    SerialEnable,
    SerialDisable,
    SerialWrite { bytes: Vec<u8> },
    
    // Capture
    Screenshot { path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    fn from_str(s: &str) -> Result<Button, String> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Button::A),
            "B" => Ok(Button::B),
            "START" => Ok(Button::Start),
            "SELECT" => Ok(Button::Select),
            "UP" => Ok(Button::Up),
            "DOWN" => Ok(Button::Down),
            "LEFT" => Ok(Button::Left),
            "RIGHT" => Ok(Button::Right),
            _ => Err(format!("Unknown button: {}", s)),
        }
    }
    
    fn to_bit(&self) -> u8 {
        match self {
            Button::Right | Button::A => 0x01,
            Button::Left  | Button::B => 0x02,
            Button::Up    | Button::Select => 0x04,
            Button::Down  | Button::Start => 0x08,
        }
    }
}

struct ScriptParser;

impl ScriptParser {
    fn parse_file(path: &Path) -> Result<Vec<Command>, String> {
        let file = fs::File::open(path)
            .map_err(|e| format!("Failed to open script: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut commands = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Read error at line {}: {}", line_num + 1, e))?;
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            match Self::parse_line(line) {
                Ok(cmd) => commands.push(cmd),
                Err(e) => return Err(format!("Line {}: {}", line_num + 1, e)),
            }
        }
        
        Ok(commands)
    }
    
    fn parse_line(line: &str) -> Result<Command, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }
        
        let cmd = parts[0].to_uppercase();
        
        match cmd.as_str() {
            "WAIT" => {
                let cycles = parts.get(1)
                    .ok_or("WAIT requires cycles argument")?
                    .parse::<usize>()
                    .map_err(|_| "WAIT cycles must be a number")?;
                Ok(Command::Wait { cycles })
            }
            
            "WAIT_FRAMES" => {
                let frames = parts.get(1)
                    .ok_or("WAIT_FRAMES requires frames argument")?
                    .parse::<usize>()
                    .map_err(|_| "WAIT_FRAMES frames must be a number")?;
                Ok(Command::WaitFrames { frames })
            }
            
            "HALT" => Ok(Command::Halt),
            
            "TIMEOUT" => {
                let cycles = parts.get(1)
                    .ok_or("TIMEOUT requires cycles argument")?
                    .parse::<usize>()
                    .map_err(|_| "TIMEOUT cycles must be a number")?;
                Ok(Command::Timeout { cycles })
            }
            
            "LOG" => {
                let message = parts[1..].join(" ");
                Ok(Command::Log { message })
            }
            
            "INPUT" => {
                let button_spec = parts.get(1)
                    .ok_or("INPUT requires buttons argument")?;
                let buttons = Self::parse_buttons(button_spec)?;
                Ok(Command::Input { buttons })
            }
            
            "RELEASE" => {
                if parts.len() > 1 && parts[1].to_uppercase() == "ALL" {
                    Ok(Command::ReleaseAll)
                } else {
                    let button_spec = parts.get(1)
                        .ok_or("RELEASE requires buttons argument or ALL")?;
                    let buttons = Self::parse_buttons(button_spec)?;
                    Ok(Command::Release { buttons })
                }
            }
            
            "SERIAL_ENABLE" => Ok(Command::SerialEnable),
            "SERIAL_DISABLE" => Ok(Command::SerialDisable),
            
            "SERIAL_WRITE" => {
                let bytes: Result<Vec<u8>, _> = parts[1..]
                    .iter()
                    .map(|s| {
                        let hex = s.trim_start_matches("0x");
                        u8::from_str_radix(hex, 16)
                    })
                    .collect();
                Ok(Command::SerialWrite {
                    bytes: bytes.map_err(|_| "SERIAL_WRITE bytes must be hex (0xAB or AB)")?
                })
            }
            
            "SCREENSHOT" => {
                let path = parts.get(1)
                    .ok_or("SCREENSHOT requires path argument")?
                    .to_string();
                Ok(Command::Screenshot { path })
            }
            
            _ => Err(format!("Unknown command: {}", cmd)),
        }
    }
    
    fn parse_buttons(spec: &str) -> Result<Vec<Button>, String> {
        spec.split('+')
            .map(|s| Button::from_str(s.trim()))
            .collect()
    }
}

struct ScriptRunner {
    gb: GameBoy,
    commands: Vec<Command>,
    timeout: usize,
    cycles: usize,
    
    // Serial I/O state
    serial_enabled: bool,
    input_queue: VecDeque<u8>,
    stdin_exhausted: bool,
    last_serial_len: usize,
    
    // Joypad state
    buttons_pressed: u8,
}

impl ScriptRunner {
    fn new(rom: Vec<u8>, commands: Vec<Command>) -> Self {
        let output_buffer = Arc::new(Mutex::new(Output::new()));
        let gb = GameBoy::new_skip_boot(rom, output_buffer);
        
        ScriptRunner {
            gb,
            commands,
            timeout: DEFAULT_TIMEOUT,
            cycles: 0,
            serial_enabled: false,
            input_queue: VecDeque::new(),
            stdin_exhausted: false,
            last_serial_len: 0,
            buttons_pressed: 0,
        }
    }
    
    fn run(&mut self) -> Result<(), String> {
        eprintln!("ROM Runner: Executing {} commands", self.commands.len());
        
        for (idx, command) in self.commands.clone().iter().enumerate() {
            if self.cycles >= self.timeout {
                return Err(format!("Timeout reached ({} cycles) at command {}", self.timeout, idx + 1));
            }
            
            self.execute_command(command, idx + 1)?;
        }
        
        eprintln!("Script complete after {} cycles", self.cycles);
        Ok(())
    }
    
    fn execute_command(&mut self, command: &Command, line_num: usize) -> Result<(), String> {
        match command {
            Command::Wait { cycles } => {
                eprintln!("[{}] WAIT {} cycles", line_num, cycles);
                self.run_cycles(*cycles)?;
            }
            
            Command::WaitFrames { frames } => {
                let cycles = frames * CYCLES_PER_FRAME;
                eprintln!("[{}] WAIT_FRAMES {} ({} cycles)", line_num, frames, cycles);
                self.run_cycles(cycles)?;
            }
            
            Command::Halt => {
                eprintln!("[{}] HALT", line_num);
                return Ok(()); // Stop execution
            }
            
            Command::Timeout { cycles } => {
                eprintln!("[{}] TIMEOUT set to {} cycles", line_num, cycles);
                self.timeout = *cycles;
            }
            
            Command::Log { message } => {
                eprintln!("[{}] LOG: {}", line_num, message);
            }
            
            Command::Input { buttons } => {
                eprintln!("[{}] INPUT {:?}", line_num, buttons);
                for button in buttons {
                    self.buttons_pressed |= button.to_bit();
                }
                self.update_joypad();
            }
            
            Command::Release { buttons } => {
                eprintln!("[{}] RELEASE {:?}", line_num, buttons);
                for button in buttons {
                    self.buttons_pressed &= !button.to_bit();
                }
                self.update_joypad();
            }
            
            Command::ReleaseAll => {
                eprintln!("[{}] RELEASE ALL", line_num);
                self.buttons_pressed = 0;
                self.update_joypad();
            }
            
            Command::SerialEnable => {
                eprintln!("[{}] SERIAL_ENABLE", line_num);
                self.serial_enabled = true;
                self.setup_stdin()?;
            }
            
            Command::SerialDisable => {
                eprintln!("[{}] SERIAL_DISABLE", line_num);
                self.serial_enabled = false;
            }
            
            Command::SerialWrite { bytes } => {
                eprintln!("[{}] SERIAL_WRITE {} bytes", line_num, bytes.len());
                for byte in bytes {
                    self.input_queue.push_back(*byte);
                }
            }
            
            Command::Screenshot { path } => {
                eprintln!("[{}] SCREENSHOT {}", line_num, path);
                self.save_screenshot(path)?;
            }
        }
        
        Ok(())
    }
    
    fn run_cycles(&mut self, target_cycles: usize) -> Result<(), String> {
        let start_cycles = self.cycles;
        
        while self.cycles - start_cycles < target_cycles {
            if self.cycles >= self.timeout {
                return Err(format!("Timeout reached ({} cycles)", self.timeout));
            }
            
            // Handle serial I/O if enabled
            if self.serial_enabled {
                self.process_serial();
            }
            
            // Feed input from queue to GB
            if !self.input_queue.is_empty() {
                if let Some(byte) = self.input_queue.pop_front() {
                    self.gb.push_serial_input(byte);
                }
            }
            
            // Execute one instruction
            let opex = self.gb.tick();
            let tick_cycles = opex.t_1 - opex.t_0;
            
            for _ in 0..tick_cycles {
                self.gb.video_cycle();
                self.gb.timer_cycle();
            }
            
            self.cycles += tick_cycles as usize;
        }
        
        Ok(())
    }
    
    fn setup_stdin(&mut self) -> Result<(), String> {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let stdin_fd = io::stdin().as_raw_fd();
            unsafe {
                let flags = libc::fcntl(stdin_fd, libc::F_GETFL);
                libc::fcntl(stdin_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
        }
        Ok(())
    }
    
    fn process_serial(&mut self) {
        if !self.stdin_exhausted {
            // Try to read from stdin
            let mut stdin = io::stdin();
            let mut chunk = [0u8; 256];
            
            match stdin.read(&mut chunk) {
                Ok(0) => self.stdin_exhausted = true,
                Ok(n) => {
                    for i in 0..n {
                        self.input_queue.push_back(chunk[i]);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => eprintln!("Warning: stdin error: {}", e),
            }
        }
        
        // Write new output to stdout
        let serial_out = self.gb.serial_output();
        if serial_out.len() > self.last_serial_len {
            let new_bytes = &serial_out[self.last_serial_len..];
            let mut stdout = io::stdout();
            stdout.write_all(new_bytes).ok();
            stdout.flush().ok();
            self.last_serial_len = serial_out.len();
        }
    }
    
    fn update_joypad(&mut self) {
        self.gb.set_joypad(self.buttons_pressed);
    }
    
    fn save_screenshot(&self, path: &str) -> Result<(), String> {
        // Access the output buffer and save the display image
        let output_buffer = self.gb.output_buffer
            .lock()
            .map_err(|e| format!("Failed to lock output buffer: {}", e))?;
        
        // Save the display image as PNG
        output_buffer.display
            .save(path)
            .map_err(|e| format!("Failed to save screenshot to {}: {}", path, e))?;
        
        eprintln!("  Saved screenshot to {}", path);
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <rom.gb> [--script <file>] [--serial]", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --script <file>   Execute script file");
        eprintln!("  --serial          Enable stdin/stdout serial I/O (legacy mode)");
        std::process::exit(1);
    }
    
    let rom_path = &args[1];
    let rom = fs::read(rom_path).expect("Failed to read ROM");
    
    // Parse command line
    let mut script_path: Option<String> = None;
    let mut serial_mode = false;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--script" => {
                script_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--serial" => {
                serial_mode = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }
    
    // Load commands
    let commands = if let Some(path) = script_path {
        ScriptParser::parse_file(Path::new(&path))
            .unwrap_or_else(|e| {
                eprintln!("Script error: {}", e);
                std::process::exit(1);
            })
    } else if serial_mode {
        // Legacy serial mode - equivalent to SERIAL_ENABLE + big timeout
        vec![
            Command::SerialEnable,
            Command::Timeout { cycles: 20_000_000 },
            Command::Wait { cycles: 20_000_000 },
        ]
    } else {
        eprintln!("Error: Must specify --script or --serial");
        std::process::exit(1);
    };
    
    // Run
    let mut runner = ScriptRunner::new(rom, commands);
    
    if let Err(e) = runner.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
