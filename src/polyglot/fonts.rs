//! Bitmap font loading for filename labels.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Result of looking up a character glyph.
pub struct GlyphLookup<'a> {
    pub glyph: &'a Vec<Vec<bool>>,
    pub is_space: bool,
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
    /// 3. Specific substitutions (e.g. " ↔ curly quotes, × → x, dashes → -)
    /// 4. Generic fallback characters: …, _, ., ?
    /// 5. Space (with skip_kerning = true)
    pub fn get_glyph(&self, c: char) -> Option<GlyphLookup<'_>> {
        // 1. Try exact character
        if let Some(g) = self.glyphs.get(&c) {
            return Some(GlyphLookup { glyph: g, is_space: c == ' ' });
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
                return Some(GlyphLookup { glyph: g, is_space: false });
            }
        }

        // 3. Try specific character substitutions for visually similar alternatives
        let specific_fallbacks: &[char] = match c {
            '"'      => &['\u{201D}', '\u{201C}'],  // straight double quote → curly right/left
            '\u{201C}' | '\u{201D}' => &['"'],       // curly double quotes → straight
            '\''     => &['\u{2019}', '\u{2018}'],   // straight single quote → curly right/left
            '\u{2018}' | '\u{2019}' => &['\''],      // curly single quotes → straight
            '\u{00D7}' => &['x'],                    // × multiplication sign → x
            '\u{2013}' | '\u{2014}' => &['-'],       // en-dash / em-dash → hyphen
            _ => &[],
        };
        for &fallback in specific_fallbacks {
            if let Some(g) = self.glyphs.get(&fallback) {
                return Some(GlyphLookup { glyph: g, is_space: false });
            }
        }

        // 4. Try generic fallback characters: …, _, ., ?
        for fallback in ['…', '_', '.', '?'] {
            if let Some(g) = self.glyphs.get(&fallback) {
                return Some(GlyphLookup { glyph: g, is_space: false });
            }
        }

        // 5. Return space (skip kerning for inserted spaces)
        self.glyphs.get(&' ').map(|g| GlyphLookup { glyph: g, is_space: true })
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

                // Skip if glyph would be out of bounds
                if glyph_x + meta.w > img_width || glyph_y + meta.h > img_height {
                    continue;
                }

                let mut glyph = Vec::with_capacity(meta.h);
                for py in 0..meta.h {
                    let mut row = Vec::with_capacity(meta.w);
                    for px in 0..meta.w {
                        let x = glyph_x + px;
                        let y = glyph_y + py;
                        // Pixel is "on" if it's dark (< 128)
                        let pixel = img.get_pixel(x as u32, y as u32).0[0];
                        row.push(pixel < 128);
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
static SKY_PNG: &[u8] = include_bytes!("../text/sky.png");
static SKY_JSON: &str = include_str!("../text/sky.json");

static SUGIMORI_PNG: &[u8] = include_bytes!("../text/sugimori.png");
static SUGIMORI_JSON: &str = include_str!("../text/sugimori.json");

static MINI_PNG: &[u8] = include_bytes!("../text/mini.png");
static MINI_JSON: &str = include_str!("../text/mini.json");

static MICRO_PNG: &[u8] = include_bytes!("../text/micro.png");
static MICRO_JSON: &str = include_str!("../text/micro.json");

/// Sky font (9×10 pixels) - for data ≤ 128 KiB
pub static SKY: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SKY_PNG, SKY_JSON));

/// Sugimori font (8×8 pixels) - for data ≤ 512 KiB
pub static SUGIMORI: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SUGIMORI_PNG, SUGIMORI_JSON));

/// Mini font (3×6 pixels) - for data ≤ 1 MiB
pub static MINI: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(MINI_PNG, MINI_JSON));

/// Micro font (3×3 pixels) - for data ≤ 3 MiB
pub static MICRO: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(MICRO_PNG, MICRO_JSON));

/// Data size thresholds for font selection.
pub const SKY_THRESHOLD: usize = 128 * 1024;      // 128 KiB
pub const SUGIMORI_THRESHOLD: usize = 512 * 1024; // 512 KiB
pub const MINI_THRESHOLD: usize = 1024 * 1024;    // 1 MiB
pub const MICRO_THRESHOLD: usize = 3 * 1024 * 1024; // 3 MiB

/// Select the appropriate font based on total data size.
/// Returns None if data is too large for labels.
pub fn select_font(total_size: usize) -> Option<&'static BitmapFont> {
    if total_size <= SKY_THRESHOLD {
        Some(&*SKY)
    } else if total_size <= SUGIMORI_THRESHOLD {
        Some(&*SUGIMORI)
    } else if total_size <= MINI_THRESHOLD {
        Some(&*MINI)
    } else if total_size <= MICRO_THRESHOLD {
        Some(&*MICRO)
    } else {
        None // Too large, no labels
    }
}
