use super::GameBoy;

use image::{GenericImage, GenericImageView};

// 114 M-cycles per scanline (= 456 T-cycles / 4). self.t counts M-cycles.
const CYCLES_PER_LINE: u64 = 114;

/// Game Boy video memory state
pub struct VideoData {
    pub t: u64,
    vram: [u8; 0x2000],
    // background palette register
    bgp: u8,
    // object palette 0
    obp0: u8,
    // object palette 1
    obp1: u8,
    // background scroll/offset x and y
    scx: u8,
    scy: u8,
    // LCD control register
    lcdc: u8,
    // LCD Y draw line
    ly: u8,
}

const GB_WIDTH: u8 = 160;
const GB_HEIGHT: u8 = 144;

impl VideoData {
    pub fn new() -> Self {
        Self {
            t: 0,
            vram: [0u8; 0x2000],
            bgp: 0xFC,  // DMG default: 11 11 11 00
            obp0: 0xFF, // DMG default
            obp1: 0xFF, // DMG default
            scx: 0x00,
            scy: 0x00,
            lcdc: 0x00,
            ly: 0x00,
        }
    }
}

pub trait VideoController {
    fn video_cycle(&mut self);
    fn vram(&self, index: usize) -> u8;
    fn set_vram(&mut self, index: usize, value: u8);
    fn oam(&self, index: usize) -> u8;
    fn bgp(&self) -> u8;
    fn set_bgp(&mut self, value: u8);
    fn obp0(&self) -> u8;
    fn set_obp0(&mut self, value: u8);
    fn obp1(&self) -> u8;
    fn set_obp1(&mut self, value: u8);
    fn scy(&self) -> u8;
    fn set_scy(&mut self, value: u8);
    fn scx(&self) -> u8;
    fn set_scx(&mut self, value: u8);
    fn lcdc(&self) -> u8;
    fn set_lcdc(&mut self, value: u8);
    fn ly(&self) -> u8;
    fn set_ly(&mut self, value: u8);
    fn draw_output(&mut self);
}

impl VideoController for GameBoy {
    fn video_cycle(&mut self) {
        self.vid.t += 1;
        let old_ly = self.vid.ly;
        self.vid.ly = ((self.vid.t / CYCLES_PER_LINE) % u64::from(GB_HEIGHT + 10)) as u8;

        // Trigger VBlank interrupt when LY transitions to 144 (start of vblank period)
        if old_ly != GB_HEIGHT && self.vid.ly == GB_HEIGHT {
            use super::cpu::CPUController;
            let ift = self.ift();
            self.set_ift(ift | 0b00001); // Set VBlank interrupt flag
        }

        // after vblank, draw
        if 0 == self.vid.ly && self.vid.t.is_multiple_of(CYCLES_PER_LINE) {
            self.draw_output();
        }
    }

    fn vram(&self, index: usize) -> u8 {
        self.vid.vram[index]
    }

    fn set_vram(&mut self, index: usize, value: u8) {
        // println!("    ; vram[0x{:02X}] = 0x{:02X}", index, value);
        self.vid.vram[index] = value;
    }

    fn draw_output(&mut self) {
        // redraw display because vram was touched!
        let (mut display, mut bg_0, mut tiles, mut bgp) = {
            let output_buffer = self
                .output_buffer
                .lock()
                .expect("output buffer mutex poisoned");
            (
                output_buffer.display.clone(),
                output_buffer.bg_0.clone(),
                output_buffer.tiles.clone(),
                output_buffer.bgp.clone(),
            )
        };

        // draw background palettes
        // GB palette values are darkness (0=white, 3=black), invert to RGB brightness
        let bgp_val = self.bgp();
        let bgp_a = (bgp_val & 0b1100_0000) >> 6;
        let bgp_a_color = image::Rgba([
            (3 - bgp_a) * 0b0101_0101,
            (3 - bgp_a) * 0b0101_0101,
            (3 - bgp_a) * 0b0101_0101,
            0xFF,
        ]);
        let bgp_b = (bgp_val & 0b0011_0000) >> 4;
        let bgp_b_color = image::Rgba([
            (3 - bgp_b) * 0b0101_0101,
            (3 - bgp_b) * 0b0101_0101,
            (3 - bgp_b) * 0b0101_0101,
            0xFF,
        ]);
        let bgp_c = (bgp_val & 0b0000_1100) >> 2;
        let bgp_c_color = image::Rgba([
            (3 - bgp_c) * 0b0101_0101,
            (3 - bgp_c) * 0b0101_0101,
            (3 - bgp_c) * 0b0101_0101,
            0xFF,
        ]);
        let bgp_d = bgp_val & 0b0000_0011;
        let bgp_d_color = image::Rgba([
            (3 - bgp_d) * 0b0101_0101,
            (3 - bgp_d) * 0b0101_0101,
            (3 - bgp_d) * 0b0101_0101,
            0xFF,
        ]);
        bgp.put_pixel(0, 0, bgp_a_color);
        bgp.put_pixel(1, 0, bgp_b_color);
        bgp.put_pixel(2, 0, bgp_c_color);
        bgp.put_pixel(3, 0, bgp_d_color);
        let bg_palette = [bgp_a_color, bgp_b_color, bgp_c_color, bgp_d_color];

        // draw tiles into debug buffer
        for i in 0..256 {
            let tile_data = &self.vid.vram[i * 16..(i + 1) * 16];
            let mut new_tile_data = [0u8; 16];
            new_tile_data.clone_from_slice(tile_data);
            for y_offset in 0..8 {
                let low_byte = new_tile_data[y_offset * 2 + 1];
                let high_byte = new_tile_data[y_offset * 2];
                let first_byte = (high_byte & 0b1000_0000)
                    | ((low_byte & 0b1000_0000) >> 1)
                    | ((high_byte & 0b0100_0000) >> 1)
                    | ((low_byte & 0b0100_0000) >> 2)
                    | ((high_byte & 0b0010_0000) >> 2)
                    | ((low_byte & 0b0010_0000) >> 3)
                    | ((high_byte & 0b0001_0000) >> 3)
                    | ((low_byte & 0b0001_0000) >> 4);
                let second_byte = ((high_byte & 0b0000_1000) << 4)
                    | ((low_byte & 0b0000_1000) << 3)
                    | ((high_byte & 0b0000_0100) << 3)
                    | ((low_byte & 0b0000_0100) << 2)
                    | ((high_byte & 0b0000_0010) << 2)
                    | ((low_byte & 0b0000_0010) << 1)
                    | ((high_byte & 0b0000_0001) << 1)
                    | (low_byte & 0b0000_0001);
                new_tile_data[y_offset * 2] = first_byte;
                new_tile_data[y_offset * 2 + 1] = second_byte;
            }

            for (j, &byte_val) in new_tile_data.iter().enumerate() {
                let tile_col = (i % 16) as u32;
                let x_tile_offset = 8 * i64::from(tile_col);
                let x = ((x_tile_offset + 4 * (j % 2) as i64) % 256) as u32;

                let tile_row = (i / 16) as u32;
                let y_tile_offset = 8 * i64::from(tile_row);
                let y = ((y_tile_offset + (j / 2) as i64) % 256) as u32;

                let byte = byte_val;
                let a = (byte & 0b1100_0000) >> 6;
                let a_color =
                    image::Rgba([(3 - a) * 0b0101_0101, (3 - a) * 0b0101_0101, (3 - a) * 0b0101_0101, 0xFF]);
                let b = (byte & 0b0011_0000) >> 4;
                let b_color =
                    image::Rgba([(3 - b) * 0b0101_0101, (3 - b) * 0b0101_0101, (3 - b) * 0b0101_0101, 0xFF]);
                let c = (byte & 0b0000_1100) >> 2;
                let c_color =
                    image::Rgba([(3 - c) * 0b0101_0101, (3 - c) * 0b0101_0101, (3 - c) * 0b0101_0101, 0xFF]);
                let d = byte & 0b0000_0011;
                let d_color =
                    image::Rgba([(3 - d) * 0b0101_0101, (3 - d) * 0b0101_0101, (3 - d) * 0b0101_0101, 0xFF]);

                tiles.put_pixel(x + tile_col, y + tile_row, a_color);
                tiles.put_pixel(x + 1 + tile_col, y + tile_row, b_color);
                tiles.put_pixel(x + 2 + tile_col, y + tile_row, c_color);
                tiles.put_pixel(x + 3 + tile_col, y + tile_row, d_color);
            }
        }

        // draw background
        for i in 0..1024 {
            let tile_index = self.vid.vram[0x1800 + i];
            let tile_data_index = tile_index as usize * 16;
            let tile_data = &self.vid.vram[tile_data_index..tile_data_index + 16];

            let mut new_tile_data = [0u8; 16];
            new_tile_data.clone_from_slice(tile_data);

            for y_offset in 0..8 {
                // XXX: this is really dumb.
                // why are you converting back into bytes?
                // convert into pixels.
                let low_byte = new_tile_data[y_offset * 2 + 1];
                let high_byte = new_tile_data[y_offset * 2];
                let first_byte = (high_byte & 0b1000_0000)
                    | ((low_byte & 0b1000_0000) >> 1)
                    | ((high_byte & 0b0100_0000) >> 1)
                    | ((low_byte & 0b0100_0000) >> 2)
                    | ((high_byte & 0b0010_0000) >> 2)
                    | ((low_byte & 0b0010_0000) >> 3)
                    | ((high_byte & 0b0001_0000) >> 3)
                    | ((low_byte & 0b0001_0000) >> 4);
                let second_byte = ((high_byte & 0b0000_1000) << 4)
                    | ((low_byte & 0b0000_1000) << 3)
                    | ((high_byte & 0b0000_0100) << 3)
                    | ((low_byte & 0b0000_0100) << 2)
                    | ((high_byte & 0b0000_0010) << 2)
                    | ((low_byte & 0b0000_0010) << 1)
                    | ((high_byte & 0b0000_0001) << 1)
                    | (low_byte & 0b0000_0001);
                new_tile_data[y_offset * 2] = first_byte;
                new_tile_data[y_offset * 2 + 1] = second_byte;
            }

            for (j, &byte_val) in new_tile_data.iter().enumerate() {
                let tile_col = (i % 32) as u32;
                let x_tile_offset = 8 * i64::from(tile_col);
                let x = ((x_tile_offset + 4 * (j % 2) as i64) % 256) as u32;
                let scrolled_x = (x + 256 - u32::from(self.scx())) % 256;

                let tile_row = (i / 32) as u32;
                let y_tile_offset = 8 * i64::from(tile_row);
                let y = ((y_tile_offset + (j / 2) as i64) % 256) as u32;
                let scrolled_y = (y + 256 - u32::from(self.scy())) % 256;

                let byte = byte_val;
                let a = (byte & 0b1100_0000) >> 6;
                let a_color = bg_palette[a as usize];
                let b = (byte & 0b0011_0000) >> 4;
                let b_color = bg_palette[b as usize];
                let c = (byte & 0b0000_1100) >> 2;
                let c_color = bg_palette[c as usize];
                let d = byte & 0b0000_0011;
                let d_color = bg_palette[d as usize];

                bg_0.put_pixel(x % 256, y % 256, a_color);
                bg_0.put_pixel((x + 1) % 256, y % 256, b_color);
                bg_0.put_pixel((x + 2) % 256, y % 256, c_color);
                bg_0.put_pixel((x + 3) % 256, y % 256, d_color);

                if scrolled_y < GB_HEIGHT.into() {
                    if (scrolled_x as u8) < GB_WIDTH {
                        display.put_pixel(scrolled_x % 256, scrolled_y, a_color);
                    }
                    if ((scrolled_x + 1) as u8) < GB_WIDTH {
                        display.put_pixel((scrolled_x + 1) % 256, scrolled_y, b_color);
                    }
                    if ((scrolled_x + 2) as u8) < GB_WIDTH {
                        display.put_pixel((scrolled_x + 2) % 256, scrolled_y, c_color);
                    }
                    if ((scrolled_x + 3) as u8) < GB_WIDTH {
                        display.put_pixel((scrolled_x + 3) % 256, scrolled_y, d_color);
                    }
                }
            }
        }

        // Draw sprites (objects) if enabled
        let lcdc = self.lcdc();
        let sprites_enabled = (lcdc & 0b00000010) != 0; // LCDC bit 1: OBJ enable
        let sprite_size = if (lcdc & 0b00000100) != 0 { 16 } else { 8 }; // LCDC bit 2: 8x8 or 8x16

        if sprites_enabled {
            // Parse all 40 sprite entries from OAM
            let mut sprites: Vec<(u8, u8, u8, u8)> = Vec::new();
            for i in 0..40 {
                let base = i * 4;
                let y = self.oam(base);
                let x = self.oam(base + 1);
                let tile_num = self.oam(base + 2);
                let attrs = self.oam(base + 3);
                
                // Skip off-screen sprites
                if y == 0 || y >= 160 || x == 0 || x >= 168 {
                    continue;
                }
                
                sprites.push((y, x, tile_num, attrs));
            }

            // Sort by X coordinate (lower X = higher priority, drawn last = on top)
            sprites.sort_by_key(|s| s.1);

            // Build object palettes
            let obp0 = self.obp0();
            let obp0_palette = [
                // Color 0 is transparent for sprites
                image::Rgba([0, 0, 0, 0]),
                image::Rgba([(3 - ((obp0 & 0b0000_1100) >> 2)) * 0b0101_0101, (3 - ((obp0 & 0b0000_1100) >> 2)) * 0b0101_0101, (3 - ((obp0 & 0b0000_1100) >> 2)) * 0b0101_0101, 0xFF]),
                image::Rgba([(3 - ((obp0 & 0b0011_0000) >> 4)) * 0b0101_0101, (3 - ((obp0 & 0b0011_0000) >> 4)) * 0b0101_0101, (3 - ((obp0 & 0b0011_0000) >> 4)) * 0b0101_0101, 0xFF]),
                image::Rgba([(3 - ((obp0 & 0b1100_0000) >> 6)) * 0b0101_0101, (3 - ((obp0 & 0b1100_0000) >> 6)) * 0b0101_0101, (3 - ((obp0 & 0b1100_0000) >> 6)) * 0b0101_0101, 0xFF]),
            ];
            let obp1 = self.obp1();
            let obp1_palette = [
                image::Rgba([0, 0, 0, 0]),
                image::Rgba([(3 - ((obp1 & 0b0000_1100) >> 2)) * 0b0101_0101, (3 - ((obp1 & 0b0000_1100) >> 2)) * 0b0101_0101, (3 - ((obp1 & 0b0000_1100) >> 2)) * 0b0101_0101, 0xFF]),
                image::Rgba([(3 - ((obp1 & 0b0011_0000) >> 4)) * 0b0101_0101, (3 - ((obp1 & 0b0011_0000) >> 4)) * 0b0101_0101, (3 - ((obp1 & 0b0011_0000) >> 4)) * 0b0101_0101, 0xFF]),
                image::Rgba([(3 - ((obp1 & 0b1100_0000) >> 6)) * 0b0101_0101, (3 - ((obp1 & 0b1100_0000) >> 6)) * 0b0101_0101, (3 - ((obp1 & 0b1100_0000) >> 6)) * 0b0101_0101, 0xFF]),
            ];

            // Draw sprites
            for (y_pos, x_pos, tile_num, attrs) in sprites {
                let palette = if (attrs & 0b00010000) != 0 { &obp1_palette } else { &obp0_palette };
                let flip_x = (attrs & 0b00100000) != 0;
                let flip_y = (attrs & 0b01000000) != 0;

                // Sprite position is offset by 16,8
                let screen_y = y_pos.wrapping_sub(16);
                let screen_x = x_pos.wrapping_sub(8);

                // Get tile data
                let tile_data_index = (tile_num as usize) * 16;
                let tile_data = &self.vid.vram[tile_data_index..tile_data_index + 16];

                // Draw 8x8 (or 8x16 if that mode is enabled)
                for ty in 0..sprite_size {
                    let row_index = if flip_y { (sprite_size - 1 - ty) as usize } else { ty as usize };
                    let low_byte = tile_data[row_index * 2 + 1];
                    let high_byte = tile_data[row_index * 2];

                    for tx in 0..8 {
                        let bit_pos = if flip_x { tx } else { 7 - tx };
                        let color_num = (((high_byte >> bit_pos) & 1) << 1) | ((low_byte >> bit_pos) & 1);

                        // Color 0 is transparent
                        if color_num == 0 {
                            continue;
                        }

                        let pixel_y = screen_y.wrapping_add(ty);
                        let pixel_x = screen_x.wrapping_add(tx);

                        // Only draw if on screen
                        if pixel_y < GB_HEIGHT && pixel_x < GB_WIDTH {
                            // TODO: sprite attribute bit 7 (OBJ-to-BG priority) not implemented.
                            // When bit 7=1, sprite should be behind BG colors 1-3.
                            // Implementing requires saving BG color indices per pixel.
                            let color = palette[color_num as usize];
                            display.put_pixel(pixel_x as u32, pixel_y as u32, color);
                        }
                    }
                }
            }
        }

        // Draw border around active background in debug buffer.
        let border_width: i16 = 12;
        for dy in -border_width..border_width {
            let dya: u8 = (if dy > 0 { dy + 1 } else { -dy }) as u8;
            let dyp = if dy < 0 {
                dy
            } else {
                dy + i16::from(GB_HEIGHT)
            };
            let y = u32::from((i16::from(self.scy()) + dyp) as u8);

            for x in 0..=0xFF {
                let mut color = bg_0.get_pixel(x, y);
                color[3] = ((color[3] as u32 * dya as u32) / (border_width as u32)) as u8;
                bg_0.put_pixel(x, y, color);
            }
        }
        for dx in -border_width..border_width {
            let dxa: u8 = (if dx > 0 { dx + 1 } else { -dx }) as u8;
            let dxp = if dx < 0 { dx } else { dx + i16::from(GB_WIDTH) };
            let x = u32::from((i16::from(self.scx()) + dxp) as u8);

            for y in 0..=0xFF {
                let mut color = bg_0.get_pixel(x, y);
                color[3] = ((color[3] as u32 * dxa as u32) / (border_width as u32)) as u8;
                bg_0.put_pixel(x, y, color);
            }
        }

        {
            let mut self_output_buffer = self
                .output_buffer
                .lock()
                .expect("output buffer mutex poisoned");
            self_output_buffer.display = display;
            self_output_buffer.bg_0 = bg_0;
            self_output_buffer.tiles = tiles;
            self_output_buffer.bgp = bgp;
        };
    }

    fn oam(&self, index: usize) -> u8 {
        use super::memory::MemoryController;
        self.mem(0xFE00 + index as u16)
    }

    fn bgp(&self) -> u8 {
        self.vid.bgp
    }

    fn set_bgp(&mut self, value: u8) {
        // println!("    ; vid bgp = 0x{:02X}", value);
        self.vid.bgp = value;
    }

    fn obp0(&self) -> u8 {
        self.vid.obp0
    }

    fn set_obp0(&mut self, value: u8) {
        self.vid.obp0 = value;
    }

    fn obp1(&self) -> u8 {
        self.vid.obp1
    }

    fn set_obp1(&mut self, value: u8) {
        self.vid.obp1 = value;
    }

    fn scy(&self) -> u8 {
        self.vid.scy
    }

    fn set_scy(&mut self, value: u8) {
        // println!("    ; vid scy = 0x{:02X}", value);
        self.vid.scy = value;
    }

    fn scx(&self) -> u8 {
        self.vid.scx
    }

    fn set_scx(&mut self, value: u8) {
        // println!("    ; vid scx = 0x{:02X}", value);
        self.vid.scx = value;
    }

    fn lcdc(&self) -> u8 {
        self.vid.lcdc
    }

    fn set_lcdc(&mut self, value: u8) {
        // println!("    ; vid lcdc = 0x{:02X}", value);
        self.vid.lcdc = value;
    }

    fn ly(&self) -> u8 {
        self.vid.ly
    }

    fn set_ly(&mut self, _value: u8) {
        // Writing any value to LY resets it to 0
        self.vid.ly = 0;
    }
}
