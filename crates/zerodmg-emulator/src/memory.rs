use super::GameBoy;

use super::audio::AudioController;
use super::cpu::CPUController;
use super::video::VideoController;

/// Game Boy general memory state
pub struct MemoryData {
    wram: [u8; 0x2000],
    stack_ram: [u8; 0x80],
    oam: [u8; 0xA0],
    io_registers: [u8; 0x80],
    boot_rom: Vec<u8>,
    game_rom: Vec<u8>,
    pub boot_rom_mapped: bool,
    // MBC1 state
    rom_bank: u8,
    ram_enable: bool,
    // Timer state
    pub div_counter: u16,  // Internal counter for DIV register (increments every cycle)
    pub tima: u8,          // Timer counter
    pub tma: u8,           // Timer modulo (reload value)
    pub tac: u8,           // Timer control
}

impl MemoryData {
    pub fn new(game_rom: Vec<u8>) -> Self {
        Self {
            wram: [0u8; 0x2000],
            stack_ram: [0u8; 0x80],
            oam: [0u8; 0xA0],
            io_registers: [0u8; 0x80],
            game_rom,
            boot_rom: zerodmg_codes::roms::dmg_boot().to_bytes(),
            boot_rom_mapped: true,
            rom_bank: 1,
            ram_enable: false,
            div_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }
}

pub trait MemoryController {
    fn mem(&self, addr: u16) -> u8;
    fn set_mem(&mut self, addr: u16, value: u8);
}

impl MemoryController for GameBoy {
    fn mem(&self, addr: u16) -> u8 {
        match addr {
            // Boot ROM, until unmapped to expose initial bytes of game ROM
            0x0000..=0x00FF if self.mem.boot_rom_mapped => self.mem.boot_rom[addr as usize],
            // Game ROM bank 0 (fixed)
            0x0000..=0x3FFF => {
                if (addr as usize) < self.mem.game_rom.len() {
                    self.mem.game_rom[addr as usize]
                } else {
                    0xFF
                }
            }
            // Game ROM bank N (switchable via MBC)
            0x4000..=0x7FFF => {
                let bank = self.mem.rom_bank.max(1) as usize;
                let rom_addr = (bank * 0x4000) + (addr as usize - 0x4000);
                if rom_addr < self.mem.game_rom.len() {
                    self.mem.game_rom[rom_addr]
                } else {
                    0xFF
                }
            }
            // Video RAM
            0x8000..=0x9FFF => {
                let i = (addr - 0x8000) as usize;
                self.vram(i)
            }
            // External RAM (cartridge) — not implemented, return 0xFF
            0xA000..=0xBFFF => 0xFF,
            // Working RAM
            0xC000..=0xDFFF => {
                let i = (addr - 0xC000) as usize;
                self.mem.wram[i]
            }
            // Echo RAM (mirror of C000-DDFF)
            0xE000..=0xFDFF => {
                let i = (addr - 0xE000) as usize;
                self.mem.wram[i]
            }
            // OAM (Sprite attribute table)
            0xFE00..=0xFE9F => {
                let i = (addr - 0xFE00) as usize;
                self.mem.oam[i]
            }
            // Unusable range
            0xFEA0..=0xFEFF => 0xFF,
            // I/O Registers
            0xFF00..=0xFF7F => self.read_io(addr),
            // High RAM (HRAM)
            0xFF80..=0xFFFE => {
                let i = (addr - 0xFF80) as usize;
                self.mem.stack_ram[i]
            }
            // Interrupt Enable
            0xFFFF => self.ie(),
        }
    }

    fn set_mem(&mut self, addr: u16, value: u8) {
        match addr {
            // MBC1 control registers
            0x0000..=0x1FFF => {
                // RAM Enable: writing 0x0A enables, anything else disables
                self.mem.ram_enable = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                // ROM Bank Number (lower 5 bits)
                let mut bank = value & 0x1F;
                if bank == 0 { bank = 1; } // bank 0 maps to bank 1
                self.mem.rom_bank = (self.mem.rom_bank & 0x60) | bank;
            }
            0x4000..=0x5FFF => {
                // RAM Bank / Upper ROM Bank bits
                self.mem.rom_bank = (self.mem.rom_bank & 0x1F) | ((value & 0x03) << 5);
            }
            0x6000..=0x7FFF => {
                // Banking Mode Select — ignored for now
            }
            // Video RAM
            0x8000..=0x9FFF => {
                let i = (addr - 0x8000) as usize;
                self.set_vram(i, value);
            }
            // External RAM (cartridge) — not implemented, ignore
            0xA000..=0xBFFF => {}
            // Working RAM
            0xC000..=0xDFFF => {
                let i = (addr - 0xC000) as usize;
                self.mem.wram[i] = value;
            }
            // Echo RAM (mirror of C000-DDFF)
            0xE000..=0xFDFF => {
                let i = (addr - 0xE000) as usize;
                self.mem.wram[i] = value;
            }
            // OAM (Sprite attribute table)
            0xFE00..=0xFE9F => {
                let i = (addr - 0xFE00) as usize;
                self.mem.oam[i] = value;
            }
            // Unusable range
            0xFEA0..=0xFEFF => {}
            // I/O Registers
            0xFF00..=0xFF7F => self.write_io(addr, value),
            // High RAM (HRAM)
            0xFF80..=0xFFFE => {
                let i = (addr - 0xFF80) as usize;
                self.mem.stack_ram[i] = value;
            }
            // Interrupt Enable
            0xFFFF => self.set_ie(value),
        }
    }
}

impl GameBoy {
    fn read_io(&self, addr: u16) -> u8 {
        match addr {
            // Joypad
            0xFF00 => 0xFF, // No buttons pressed
            // Serial Data (SB)
            0xFF01 => self.sb_register,
            // Serial Control (SC)
            0xFF02 => 0x00,
            // Timer registers
            0xFF04 => (self.mem.div_counter >> 8) as u8, // DIV (upper byte of internal counter)
            0xFF05 => self.mem.tima, // TIMA
            0xFF06 => self.mem.tma,  // TMA
            0xFF07 => self.mem.tac,  // TAC
            // Interrupt Flag
            0xFF0F => self.ift(),
            // Audio registers
            0xFF10..=0xFF26 => {
                let i = (addr - 0xFF10) as usize;
                self.audio_register(i)
            }
            // Wave pattern RAM
            0xFF30..=0xFF3F => 0x00,
            // LCD Control
            0xFF40 => self.lcdc(),
            // LCD Status
            0xFF41 => 0x00, // stub
            // Scroll Y
            0xFF42 => self.scy(),
            // Scroll X
            0xFF43 => self.scx(),
            // LCD Y-Coordinate
            0xFF44 => self.ly(),
            // LY Compare
            0xFF45 => 0x00,
            // DMA Transfer
            0xFF46 => 0x00,
            // Background Palette
            0xFF47 => self.bgp(),
            // Object Palette 0
            0xFF48 => 0x00,
            // Object Palette 1
            0xFF49 => 0x00,
            // Window Y
            0xFF4A => 0x00,
            // Window X
            0xFF4B => 0x00,
            // Boot ROM disable
            0xFF50 => if self.mem.boot_rom_mapped { 0x00 } else { 0x01 },
            // Everything else in I/O range — return from generic storage
            _ => {
                let i = (addr - 0xFF00) as usize;
                self.mem.io_registers[i]
            }
        }
    }

    fn write_io(&mut self, addr: u16, value: u8) {
        // Store in generic I/O register backing
        let i = (addr - 0xFF00) as usize;
        self.mem.io_registers[i] = value;

        // Handle specific side effects
        match addr {
            // Serial Data (SB)
            0xFF01 => self.sb_register = value,
            // Serial Control (SC) - writing 0x81 triggers transfer
            0xFF02 => {
                if value == 0x81 {
                    self.serial_output.push(self.sb_register);
                }
            }
            // Timer registers
            0xFF04 => { self.mem.div_counter = 0; } // DIV — writing resets to 0
            0xFF05 => { self.mem.tima = value; }     // TIMA
            0xFF06 => { self.mem.tma = value; }      // TMA
            0xFF07 => { self.mem.tac = value; }      // TAC
            // Interrupt Flag
            0xFF0F => self.set_ift(value),
            // Audio registers
            0xFF10..=0xFF26 => {
                let ai = (addr - 0xFF10) as usize;
                self.set_audio_register(ai, value);
            }
            // LCD Control
            0xFF40 => self.set_lcdc(value),
            // Scroll Y
            0xFF42 => self.set_scy(value),
            // Scroll X
            0xFF43 => self.set_scx(value),
            // LCD Y-Coordinate (writing resets)
            0xFF44 => self.set_ly(0),
            // Background Palette
            0xFF47 => self.set_bgp(value),
            // Boot ROM disable
            0xFF50 => {
                if value != 0 {
                    self.mem.boot_rom_mapped = false;
                }
            }
            // DMA Transfer
            0xFF46 => {
                // Copy 160 bytes from (value * 0x100) to OAM
                let source_base = (value as u16) * 0x100;
                for offset in 0..0xA0u16 {
                    let byte = self.mem(source_base + offset);
                    self.mem.oam[offset as usize] = byte;
                }
            }
            // Everything else — already stored in io_registers above
            _ => {}
        }
    }
}
