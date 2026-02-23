//! Sierpinski Triangle on Game Boy
//!
//! The classic XOR fractal: pixel (x,y) is lit if (x & y) == 0
//! This creates the Sierpinski triangle pattern.
//!
//! We draw directly to the tile data, creating a 128x128 pattern
//! using 16x16 tiles (8x8 pixels each).
//!
//! The beauty: this pattern emerges from a single bitwise AND.

mod gb_asm;
use gb_asm::*;
use zerodmg_codes::instruction::Instruction;

fn main() {
    let rom = build_rom();
    std::fs::write("sierpinski.gb", &rom).expect("Failed to write ROM");
    println!("Generated sierpinski.gb");
    println!("Sierpinski triangle via (x & y) == 0");
}

fn build_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    // Nintendo logo
    rom.extend_from_slice(&[
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
        0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
        0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
        0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
        0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
        0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    ]);
    
    // Title
    rom.extend_from_slice(&[b'S', b'I', b'E', b'R', b'P', b'I', b'N', b'S', b'K', b'I', 0, 0, 0, 0, 0, 0]);
    
    // Pad to code start
    while rom.len() < 0x0150 { rom.push(0); }
    
    // Generate code
    for inst in game_code() {
        rom.extend_from_slice(&inst.to_bytes());
    }
    
    // Pad to 32KB
    while rom.len() < 32768 { rom.push(0); }
    rom
}

fn game_code() -> Vec<Instruction> {
    use zerodmg_codes::instruction::prelude::*;
    
    let mut asm = Assembler::new();
    
    // Disable interrupts, set stack
    asm.inst(DI)
       .inst(LD_16_IMMEDIATE(SP, 0xFFFE));
    
    // Turn on LCD with BG enabled
    asm.inst(LD_8_IMMEDIATE(A, 0x91))
       .inst(LD_8_TO_FF_IMMEDIATE(0x40));  // LCDC
    
    // Set BGP (background palette): 0 = white, 3 = black
    asm.inst(LD_8_IMMEDIATE(A, 0xE4))  // 11 10 01 00 = black/dark/light/white
       .inst(LD_8_TO_FF_IMMEDIATE(0x47));  // BGP
    
    // We'll generate 256 tiles (tiles 0-255) where tile N has the Sierpinski pattern
    // for that 8x8 region. Each tile covers a different part of the 128x128 image.
    //
    // For a 16x16 tile grid (128x128 pixels):
    //   Tile at (tx, ty) covers pixels (tx*8, ty*8) to (tx*8+7, ty*8+7)
    //   Tile index = ty * 16 + tx
    //
    // For each tile, we generate 16 bytes (8 rows × 2 bytes per row).
    // For row r in tile (tx, ty):
    //   y = ty * 8 + r
    //   For column c (0-7):
    //     x = tx * 8 + c
    //     pixel_on = (x & y) == 0
    //
    // Since we can't do this dynamically (no multiply), we pre-compute in Rust
    // and just load the tile data.
    
    // Actually, let's do it the clever way: compute at runtime!
    // 
    // WRAM layout:
    //   0xC000 = current tile index (0-255)
    //   0xC001 = tx (tile X: 0-15)
    //   0xC002 = ty (tile Y: 0-15)
    //
    // For each tile:
    //   base_x = tx * 8
    //   base_y = ty * 8
    //   For row 0-7:
    //     y = base_y + row
    //     byte_lo = 0, byte_hi = 0
    //     For bit 0-7:
    //       x = base_x + bit
    //       if (x & y) == 0: set bit in byte_lo
    //     Write byte_lo, byte_hi to VRAM
    
    // Initialize tile counter
    asm.inst(LD_8_IMMEDIATE(A, 0))
       .inst(LD_16_IMMEDIATE(HL, 0xC000))
       .inst(LD_8_INTERNAL(AT_HL, A))  // tile index = 0
       .inst(INC_16(HL))
       .inst(LD_8_INTERNAL(AT_HL, A))  // tx = 0
       .inst(INC_16(HL))
       .inst(LD_8_INTERNAL(AT_HL, A)); // ty = 0
    
    // VRAM tile data starts at 0x8000
    // Each tile is 16 bytes
    asm.inst(LD_16_IMMEDIATE(DE, 0x8000));  // DE = VRAM pointer
    
    // TILE_LOOP: process 256 tiles
    asm.label("TILE_LOOP");
    
    // Load tx, ty
    asm.inst(LD_16_IMMEDIATE(HL, 0xC001))
       .inst(LD_8_INTERNAL(B, AT_HL))   // B = tx
       .inst(INC_16(HL))
       .inst(LD_8_INTERNAL(C, AT_HL));  // C = ty
    
    // Compute base_x = tx * 8 (shift left 3)
    asm.inst(LD_8_INTERNAL(A, B))
       .inst(ADD(A)).inst(ADD(A)).inst(ADD(A))  // A = tx * 8
       .inst(LD_16_IMMEDIATE(HL, 0xC010))
       .inst(LD_8_INTERNAL(AT_HL, A));  // 0xC010 = base_x
    
    // Compute base_y = ty * 8
    asm.inst(LD_8_INTERNAL(A, C))
       .inst(ADD(A)).inst(ADD(A)).inst(ADD(A))  // A = ty * 8
       .inst(INC_16(HL))
       .inst(LD_8_INTERNAL(AT_HL, A));  // 0xC011 = base_y
    
    // Process 8 rows
    asm.inst(LD_8_IMMEDIATE(B, 8));  // row counter
    
    asm.label("ROW_LOOP");
    
    // y = base_y + (8 - B) = base_y + row
    asm.inst(LD_8_IMMEDIATE(A, 8))
       .inst(SUB(B))                   // A = 8 - B = row number (0-7)
       .inst(LD_16_IMMEDIATE(HL, 0xC011))
       .inst(ADD(AT_HL))               // A = base_y + row = y
       .inst(LD_16_IMMEDIATE(HL, 0xC020))
       .inst(LD_8_INTERNAL(AT_HL, A)); // 0xC020 = y
    
    // Build the byte for this row: bit c is set if (base_x + c) & y == 0
    asm.inst(LD_8_IMMEDIATE(A, 0))
       .inst(LD_16_IMMEDIATE(HL, 0xC021))
       .inst(LD_8_INTERNAL(AT_HL, A));  // 0xC021 = result byte (starts at 0)
    
    asm.inst(LD_8_IMMEDIATE(C, 8));  // bit counter (7 down to 0)
    
    asm.label("BIT_LOOP");
    
    // x = base_x + (8 - C)
    asm.inst(LD_8_IMMEDIATE(A, 8))
       .inst(SUB(C))                   // A = bit position (0-7)
       .inst(LD_16_IMMEDIATE(HL, 0xC010))
       .inst(ADD(AT_HL))               // A = base_x + bit = x
    
    // Check if (x & y) == 0
       .inst(LD_16_IMMEDIATE(HL, 0xC020))
       .inst(AND(AT_HL));              // A = x & y
    
    // If A != 0, skip setting the bit
    asm.jr_cond(if_NZ, "SKIP_BIT");
    
    // Set bit (8-C) in result
    // We need to set bit (8-C-1) = (7-C+1) ... actually bit 0 is rightmost
    // For Sierpinski, bit 7 is leftmost pixel, bit 0 is rightmost
    // So for column c (0-7), we set bit (7-c)
    // (8-C) gives us column 0-7, so we want bit 7-(8-C) = C-1
    
    // Actually simpler: shift 1 left by (8-C-1) = (7-C+1-1) = ... 
    // Let's just build the byte by shifting: result = (result << 1) | pixel
    
    // Load result, shift left, OR with 1 if pixel on
    asm.inst(LD_16_IMMEDIATE(HL, 0xC021))
       .inst(LD_8_INTERNAL(A, AT_HL))
       .inst(ADD(A))                   // A = result << 1
       .inst(OR_IMMEDIATE(1))          // A = (result << 1) | 1
       .inst(LD_8_INTERNAL(AT_HL, A))
       .jr("NEXT_BIT");
    
    asm.label("SKIP_BIT");
    // Pixel off: just shift result left
    asm.inst(LD_16_IMMEDIATE(HL, 0xC021))
       .inst(LD_8_INTERNAL(A, AT_HL))
       .inst(ADD(A))                   // A = result << 1
       .inst(LD_8_INTERNAL(AT_HL, A));
    
    asm.label("NEXT_BIT");
    asm.inst(DEC(C))
       .jr_cond(if_NZ, "BIT_LOOP");
    
    // Write result byte to VRAM (twice for lo/hi plane — solid color)
    asm.inst(LD_16_IMMEDIATE(HL, 0xC021))
       .inst(LD_8_INTERNAL(A, AT_HL))
       .inst(LD_8_TO_SECONDARY(AT_DE))
       .inst(INC_16(DE))
       .inst(LD_8_TO_SECONDARY(AT_DE))  // Write same byte to both planes
       .inst(INC_16(DE));
    
    // Next row
    asm.inst(DEC(B))
       .jr_cond(if_NZ, "ROW_LOOP");
    
    // Tile complete. Increment tile index, tx, ty
    asm.inst(LD_16_IMMEDIATE(HL, 0xC001))
       .inst(LD_8_INTERNAL(A, AT_HL))  // A = tx
       .inst(INC(A))
       .inst(AND_IMMEDIATE(0x0F))      // tx = (tx + 1) & 15
       .inst(LD_8_INTERNAL(AT_HL, A))
       .jr_cond(if_NZ, "NO_TY_INC");
    
    // tx wrapped to 0, increment ty
    asm.inst(INC_16(HL))
       .inst(LD_8_INTERNAL(A, AT_HL))
       .inst(INC(A))
       .inst(LD_8_INTERNAL(AT_HL, A)); // ty++
    
    asm.label("NO_TY_INC");
    
    // Increment tile counter
    asm.inst(LD_16_IMMEDIATE(HL, 0xC000))
       .inst(LD_8_INTERNAL(A, AT_HL))
       .inst(INC(A))
       .inst(LD_8_INTERNAL(AT_HL, A));
    
    // Continue until all 256 tiles done (A wraps to 0)
    asm.jr_cond(if_NZ, "TILE_LOOP");
    
    // Now set up the BG tilemap to display tiles 0-255 in a 16x16 grid
    // BG map at 0x9800, 32 tiles wide
    // We want tiles arranged so tile N goes at position (N%16, N/16)
    // Offset to center the 16x16 in the 20x18 visible area: start at (2, 1)
    
    asm.inst(LD_8_IMMEDIATE(B, 0));    // tile index
    asm.inst(LD_16_IMMEDIATE(HL, 0x9800 + 1*32 + 2));  // Start position in BG map
    
    asm.label("MAP_ROW");
    asm.inst(LD_8_IMMEDIATE(C, 16));   // 16 tiles per row
    
    asm.label("MAP_COL");
    asm.inst(LD_8_INTERNAL(AT_HL, B))
       .inst(INC_16(HL))
       .inst(INC(B))
       .inst(DEC(C))
       .jr_cond(if_NZ, "MAP_COL");
    
    // Skip 16 tiles to next row (32 - 16 = 16)
    asm.inst(LD_8_INTERNAL(A, L))
       .inst(ADD_IMMEDIATE(16))
       .inst(LD_8_INTERNAL(L, A))
       .inst(LD_8_IMMEDIATE(A, 0))
       .inst(ADC(H))
       .inst(LD_8_INTERNAL(H, A));
    
    // Check if we've done all 256 tiles
    asm.inst(LD_8_INTERNAL(A, B))
       .inst(OR(A))
       .jr_cond(if_NZ, "MAP_ROW");
    
    // Done! Halt
    asm.label("DONE");
    asm.inst(HALT)
       .jr("DONE");
    
    asm.assemble()
}
