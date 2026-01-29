//! Bitmap font loading for filename labels.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Result of looking up a character glyph.
pub struct GlyphLookup<'a> {
    pub glyph: &'a Vec<Vec<bool>>,
    pub skip_kerning: bool, // True for spaces (would kern to nothing otherwise)
}

/// A loaded bitmap font with glyph data.
pub struct BitmapFont {
    pub name: &'static str,
    pub width: usize,
    pub height: usize,
    glyphs: HashMap<char, Vec<Vec<bool>>>, // [row][col] = pixel on
}

impl BitmapFont {
    /// Get glyph bitmap for a character with fallback chain:
    /// 1. Exact character
    /// 2. Different capitalization (upper ↔ lower)
    /// 3. Fallback characters: …, _, ., ?
    /// 4. Space (with skip_kerning = true)
    /// Returns the set of characters this font has native glyphs for.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.glyphs.keys().copied()
    }

    pub fn get_glyph(&self, c: char) -> Option<GlyphLookup<'_>> {
        // 1. Try exact character
        if let Some(g) = self.glyphs.get(&c) {
            return Some(GlyphLookup { glyph: g, skip_kerning: c == ' ' });
        }

        // 2. Try different capitalization
        let alt_case = if c.is_uppercase() {
            c.to_lowercase().next()
        } else if c.is_lowercase() {
            c.to_uppercase().next()
        } else {
            None
        };
        if let Some(alt) = alt_case {
            if let Some(g) = self.glyphs.get(&alt) {
                return Some(GlyphLookup { glyph: g, skip_kerning: false });
            }
        }

        // 3. Try fallback characters: …, _, ., ?
        for fallback in ['…', '_', '.', '?'] {
            if let Some(g) = self.glyphs.get(&fallback) {
                return Some(GlyphLookup { glyph: g, skip_kerning: false });
            }
        }

        // 4. Return space (skip kerning for inserted spaces)
        self.glyphs.get(&' ').map(|g| GlyphLookup { glyph: g, skip_kerning: true })
    }

    /// Compute the space bar width: half (rounded up) of the maximum ink width
    /// across all non-space glyphs. Returns at least 1.
    fn space_bar_width(&self) -> usize {
        let mut max_ink = 0usize;
        for (c, glyph) in &self.glyphs {
            if *c == ' ' { continue; }
            let mut min_col = usize::MAX;
            let mut max_col = 0;
            let mut has_pixel = false;
            for row in glyph {
                for (col, &on) in row.iter().enumerate() {
                    if on {
                        min_col = min_col.min(col);
                        max_col = max_col.max(col);
                        has_pixel = true;
                    }
                }
            }
            if has_pixel {
                max_ink = max_ink.max(max_col - min_col + 1);
            }
        }
        if max_ink == 0 { return 1; }
        (max_ink / 2).max(1)
    }

    /// Build a vertical bar glyph (full height, centered) with the given ink width.
    /// Used as a synthetic space glyph for kerning purposes.
    fn space_bar_glyph(&self, ink_width: usize) -> Vec<Vec<bool>> {
        let mut glyph = vec![vec![false; self.width]; self.height];
        // Center the bar horizontally within the glyph bounding box
        let start = (self.width.saturating_sub(ink_width)) / 2;
        for row in &mut glyph {
            for col in start..start + ink_width.min(self.width) {
                row[col] = true;
            }
        }
        glyph
    }

    /// Lay out text with canvas-based kerning.
    /// Returns `(canvas, char_positions, actual_width)` where:
    /// - `canvas` is the rendered bitmap `[row][col]`
    /// - `char_positions` maps each input char index to its x offset
    /// - `actual_width` is the width of the rendered text in pixels
    ///
    /// Space characters are kerned as if they were a vertical bar with
    /// the median ink width of the font, but are not drawn to the canvas.
    pub fn layout_text(&self, text: &str) -> (Vec<Vec<bool>>, Vec<(usize, i32)>, usize) {
        let space_bar = self.space_bar_glyph(self.space_bar_width());

        let lookups: Vec<Option<GlyphLookup<'_>>> = text.chars()
            .map(|c| self.get_glyph(c))
            .collect();

        let max_canvas_width = text.len() * self.width * 2;
        let mut kern_canvas: Vec<Vec<bool>> = vec![vec![false; max_canvas_width]; self.height];
        let mut display_canvas: Vec<Vec<bool>> = vec![vec![false; max_canvas_width]; self.height];
        let mut char_positions: Vec<(usize, i32)> = Vec::new();
        let mut total_width = 0i32;
        let mut rightmost_pixel = 0i32;

        for (i, lookup_opt) in lookups.iter().enumerate() {
            if let Some(lookup) = lookup_opt {
                let is_space = lookup.skip_kerning;
                // For spaces, kern using the vertical bar; for others, use the real glyph.
                let kern_glyph: &[Vec<bool>] = if is_space { &space_bar } else { lookup.glyph };

                let mut best_offset = total_width + self.width as i32;

                for test_offset in (total_width - self.width as i32 + 1)..=best_offset {
                    if test_offset < 0 {
                        continue;
                    }
                    let mut touches = false;
                    'check: for (gy, glyph_row) in kern_glyph.iter().enumerate() {
                        for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                            if !pixel_on {
                                continue;
                            }
                            let cx = test_offset as usize + gx;
                            for dy in -1i32..=1 {
                                for dx in -1i32..=1 {
                                    let ny = gy as i32 + dy;
                                    let nx = cx as i32 + dx;
                                    if ny >= 0 && (ny as usize) < self.height && nx >= 0 {
                                        if kern_canvas[ny as usize].get(nx as usize).copied().unwrap_or(false) {
                                            touches = true;
                                            break 'check;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !touches {
                        best_offset = test_offset;
                        break;
                    }
                }

                char_positions.push((i, best_offset));

                // Stamp the space bar onto the kern canvas so subsequent chars kern away.
                // For non-spaces, stamp the real glyph onto both canvases.
                let kern_stamp: &[Vec<bool>] = if is_space { &space_bar } else { lookup.glyph };
                for (gy, glyph_row) in kern_stamp.iter().enumerate() {
                    for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                        if pixel_on {
                            let cx = best_offset as usize + gx;
                            if cx < max_canvas_width {
                                kern_canvas[gy][cx] = true;
                            }
                        }
                    }
                }
                if !is_space {
                    for (gy, glyph_row) in lookup.glyph.iter().enumerate() {
                        for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                            if pixel_on {
                                let cx = best_offset as usize + gx;
                                if cx < max_canvas_width {
                                    display_canvas[gy][cx] = true;
                                }
                                rightmost_pixel = rightmost_pixel.max(cx as i32 + 1);
                            }
                        }
                    }
                }

                total_width = best_offset + self.width as i32;
            }
        }

        let actual_width = rightmost_pixel.max(0) as usize;
        (display_canvas, char_positions, actual_width)
    }

    /// Load font from embedded PNG and JSON metadata.
    fn load(png_data: &[u8], json_data: &str) -> Self {
        let meta: FontMeta = serde_json::from_str(json_data)
            .expect("Failed to parse font JSON");

        let img = image::load_from_memory(png_data)
            .expect("Failed to load font PNG")
            .to_luma8();

        let img_width = img.width() as usize;
        let img_height = img.height() as usize;

        let mut glyphs = HashMap::new();

        for (row_idx, row_chars) in meta.rows.iter().enumerate() {
            for (col_idx, c) in row_chars.chars().enumerate() {
                if c == ' ' {
                    continue; // Skip spaces in the character map
                }

                let glyph_x = meta.x + col_idx * meta.dx;
                let glyph_y = meta.y + row_idx * meta.dy;

                // Skip if glyph starts entirely outside the image
                if glyph_x >= img_width || glyph_y >= img_height {
                    continue;
                }

                let mut glyph = Vec::with_capacity(meta.h);
                for py in 0..meta.h {
                    let mut row = Vec::with_capacity(meta.w);
                    for px in 0..meta.w {
                        let x = glyph_x + px;
                        let y = glyph_y + py;
                        // Treat out-of-bounds pixels as white (off)
                        if x >= img_width || y >= img_height {
                            row.push(false);
                        } else {
                            let pixel = img.get_pixel(x as u32, y as u32).0[0];
                            row.push(pixel < 128);
                        }
                    }
                    glyph.push(row);
                }
                glyphs.insert(c, glyph);
            }
        }

        // Add space glyph (all off)
        let space_glyph: Vec<Vec<bool>> = (0..meta.h)
            .map(|_| vec![false; meta.w])
            .collect();
        glyphs.insert(' ', space_glyph);

        BitmapFont {
            name: Box::leak(meta.name.into_boxed_str()),
            width: meta.w,
            height: meta.h,
            glyphs,
        }
    }
}

#[derive(serde::Deserialize)]
struct FontMeta {
    x: usize,
    y: usize,
    dx: usize,
    dy: usize,
    w: usize,
    h: usize,
    #[allow(dead_code)]
    b: usize,
    rows: Vec<String>,
    name: String,
}

// Embed font data at compile time
static MICRO_PNG: &[u8] = include_bytes!("../text/micro.png");
static MICRO_JSON: &str = include_str!("../text/micro.json");

static MINI_PNG: &[u8] = include_bytes!("../text/mini.png");
static MINI_JSON: &str = include_str!("../text/mini.json");

static MONTE_PNG: &[u8] = include_bytes!("../text/monte.png");
static MONTE_JSON: &str = include_str!("../text/monte.json");

static SIXTH_PNG: &[u8] = include_bytes!("../text/sixth.png");
static SIXTH_JSON: &str = include_str!("../text/sixth.json");

static SKY_PNG: &[u8] = include_bytes!("../text/sky.png");
static SKY_JSON: &str = include_str!("../text/sky.json");

static SUGIMORI_PNG: &[u8] = include_bytes!("../text/sugimori.png");
static SUGIMORI_JSON: &str = include_str!("../text/sugimori.json");

static SWISS_PNG: &[u8] = include_bytes!("../text/swiss.png");
static SWISS_JSON: &str = include_str!("../text/swiss.json");

/// Micro font (3×3 pixels).
/// Inspired by u/Udzu's Unicase Micro.
pub static MICRO: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(MICRO_PNG, MICRO_JSON));

/// Mini font (3×6 pixels).
/// Inspired by u/Udzu's Mini.
pub static MINI: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(MINI_PNG, MINI_JSON));

/// Monte font (5×9 pixels).
/// Origin unknown.
pub static MONTE: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(MONTE_PNG, MONTE_JSON));

/// Sixth font (6×8 pixels).
/// Origin unknown.
pub static SIXTH: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SIXTH_PNG, SIXTH_JSON));

/// Sky font (9×10 pixels).
/// Inspired by Chicago, a classic Macintosh system font by Susan Kare.
pub static SKY: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SKY_PNG, SKY_JSON));

/// Sugimori font (8×8 pixels).
/// Inspired by the Pokémon Red/Blue font.
pub static SUGIMORI: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SUGIMORI_PNG, SUGIMORI_JSON));

/// Swiss font (6×8 pixels).
/// Inspired by Geneva, a classic Macintosh system font by Susan Kare.
pub static SWISS: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SWISS_PNG, SWISS_JSON));

/// All available fonts, ordered by size (smallest to largest height).
pub static ALL_FONTS: &[&Lazy<BitmapFont>] = &[
    &MICRO,    // 3×3
    &MINI,     // 3×6
    &SIXTH,    // 6×8
    &SWISS,    // 6×8
    &SUGIMORI, // 8×8
    &MONTE,    // 5×9
    &SKY,      // 9×10
];

/// Maximum data size threshold for using labels at all.
pub const MAX_LABEL_THRESHOLD: usize = 3 * 1024 * 1024; // 3 MiB

/// Font selection result: may use different fonts for filename vs file size.
pub struct FontSelection {
    /// Font used for the filename text.
    pub name_font: &'static BitmapFont,
    /// Font used for the file size text (may differ from name_font).
    pub size_font: &'static BitmapFont,
}

impl FontSelection {
    /// The taller of the two fonts, used for label height calculations.
    pub fn max_height(&self) -> usize {
        self.name_font.height.max(self.size_font.height)
    }
}

/// Select fonts based on total data size and a hash value for deterministic randomization.
/// Returns None if data is too large for labels.
///
/// - ≤512 KiB: filename uses one of SWISS/SIXTH/SKY/MONTE (hash-selected), size uses SUGIMORI
/// - ≤1 MiB: both use MINI
/// - ≤3 MiB: both use MICRO
/// - >3 MiB: no labels
pub fn select_font(total_size: usize, hash: u32) -> Option<FontSelection> {
    if total_size > MAX_LABEL_THRESHOLD {
        return None;
    }

    if total_size <= 512 * 1024 {
        // Filename: randomly select from SWISS, SIXTH, SKY, MONTE
        let name_candidates: &[&Lazy<BitmapFont>] = &[&SWISS, &SIXTH, &SKY, &MONTE];
        let index = (hash as usize) % name_candidates.len();
        Some(FontSelection {
            name_font: &name_candidates[index],
            size_font: &SUGIMORI,
        })
    } else if total_size <= 1024 * 1024 {
        Some(FontSelection {
            name_font: &MINI,
            size_font: &MINI,
        })
    } else {
        Some(FontSelection {
            name_font: &MICRO,
            size_font: &MICRO,
        })
    }
}
