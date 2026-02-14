//! Conway's Game of Life for Game Boy
//! 
//! Uses sprites to represent cells in a 20x18 grid (160x144 screen / 8x8 sprites).
//! Each sprite represents one cell (alive = visible, dead = invisible).

use zerodmg_codes::instruction::prelude::*;

const GRID_WIDTH: usize = 20;
const GRID_HEIGHT: usize = 18;
const CELLS: usize = GRID_WIDTH * GRID_HEIGHT; // 360 cells

fn main() {
    let rom = build_rom();
    std::fs::write("conway.gb", &rom).expect("Failed to write ROM");
    println!("Generated conway.gb ({} bytes)", rom.len());
    println!("Grid: {}x{} = {} cells", GRID_WIDTH, GRID_HEIGHT, CELLS);
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    // Nintendo logo and header (required for Game Boy ROMs)
    rom.extend_from_slice(&nintendo_logo());
    
    // ROM header (0x0104-0x014F)
    rom.extend_from_slice(&[
        // Title (0x0134-0x0143): "CONWAY"
        b'C', b'O', b'N', b'W', b'A', b'Y', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    
    // Skip to cartridge type at 0x0147
    while rom.len() < 0x0147 { rom.push(0); }
    rom.push(0x00); // ROM ONLY
    
    // Skip to checksum area
    while rom.len() < 0x014E { rom.push(0); }
    rom.push(0); // Header checksum (will be wrong, emulator doesn't care in skip-boot mode)
    rom.push(0); // Global checksum
    
    // Game code starts at 0x0150
    while rom.len() < 0x0150 { rom.push(0); }
    
    let instructions = game_code();
    for inst in instructions {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    // Pad to minimum ROM size (32KB)
    while rom.len() < 32768 {
        rom.push(0);
    }
    
    rom
}

fn game_code() -> Vec<Instruction> {
    use Instruction::*;
    use U8Register::*;
    use U16Register::*;
    
    vec![
        // Initialize hardware
        // Disable interrupts
        DI,
        
        // Set stack pointer
        LD_16_IMMEDIATE(SP, 0xFFFE),
        
        // Turn off LCD (required before accessing VRAM)
        LD_8_IMMEDIATE(A, 0x00),
        LD_8_TO_FF_IMMEDIATE(0x40), // LCDC = 0 (LCD off)
        
        // Load sprite tile data (tile 0: empty, tile 1: filled cell)
        // Tile data at 0x8000-0x8FFF
        // Each tile is 16 bytes (8x8 pixels, 2 bits per pixel)
        
        // HL = tile data destination (0x8010 for tile 1)
        LD_16_IMMEDIATE(HL, 0x8010),
        
        // Write filled square pattern for "alive" cell (tile 1)
        // Each row: 2 bytes (8 pixels * 2 bits/pixel)
        LD_8_IMMEDIATE(A, 0xFF), // All pixels on
        // Write 16 bytes (8 rows * 2 bytes)
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_TO_SECONDARY(AT_HL_Plus), LD_8_TO_SECONDARY(AT_HL_Plus),
        
        // Initialize OAM (sprite attribute table) at 0xFE00
        // Set up initial pattern - glider
        // Sprite format: Y, X, Tile#, Attributes
        
        // Glider pattern at center (10, 9):
        //   X
        //    X
        //  XXX
        
        LD_16_IMMEDIATE(HL, 0xFE00),
        
        // Sprite 0: (9, 11) - top of glider
        LD_8_IMMEDIATE(A, 9 * 8 + 16),  // Y position
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 11 * 8 + 8),  // X position
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 1),            // Tile 1 (filled)
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 0),            // Attributes
        LD_8_TO_SECONDARY(AT_HL_Plus),
        
        // Sprite 1: (10, 12)
        LD_8_IMMEDIATE(A, 10 * 8 + 16),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 12 * 8 + 8),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 1),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 0),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        
        // Sprite 2: (11, 10)
        LD_8_IMMEDIATE(A, 11 * 8 + 16),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 10 * 8 + 8),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 1),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 0),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        
        // Sprite 3: (11, 11)
        LD_8_IMMEDIATE(A, 11 * 8 + 16),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 11 * 8 + 8),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 1),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 0),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        
        // Sprite 4: (11, 12)
        LD_8_IMMEDIATE(A, 11 * 8 + 16),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 12 * 8 + 8),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 1),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        LD_8_IMMEDIATE(A, 0),
        LD_8_TO_SECONDARY(AT_HL_Plus),
        
        // Set sprite palettes
        LD_8_IMMEDIATE(A, 0xE4), // White sprites
        LD_8_TO_FF_IMMEDIATE(0x48), // OBP0
        
        // Turn on LCD with sprites enabled
        LD_8_IMMEDIATE(A, 0x83), // LCDC: LCD on, BG off, Sprites on (8x8)
        LD_8_TO_FF_IMMEDIATE(0x40),
        
        // Main loop - just infinite loop for now (no game logic yet)
        // Label 0x01XX where we are now
        HALT, // Wait for VBlank
        JR(-1), // Loop forever
    ]
}

fn nintendo_logo() -> [u8; 0x30] {
    // Standard Nintendo logo bytes (required for real hardware, not for emulator)
    [
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
        0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
        0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
        0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
        0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
        0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    ]
}
