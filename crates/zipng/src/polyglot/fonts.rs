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
/// Origin unknown.
pub static SKY: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SKY_PNG, SKY_JSON));

/// Sugimori font (8×8 pixels).
/// Inspired by the Pokémon Red/Blue font.
pub static SUGIMORI: Lazy<BitmapFont> = Lazy::new(|| BitmapFont::load(SUGIMORI_PNG, SUGIMORI_JSON));

/// Swiss font (6×8 pixels).
/// Possibly inspired by Susan Kare's classic Macintosh fonts.
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

/// Select a font based on total data size and a hash value for deterministic randomization.
/// Returns None if data is too large for labels.
///
/// The hash is used to pseudo-randomly select among fonts that fit within the size constraints.
/// Larger data sizes restrict which fonts can be used (smaller fonts only).
pub fn select_font(total_size: usize, hash: u32) -> Option<&'static BitmapFont> {
    if total_size > MAX_LABEL_THRESHOLD {
        return None; // Too large for labels
    }

    // Filter fonts that fit the size constraint
    // Larger fonts need more vertical space, so we limit them for larger data
    let candidates: Vec<&'static BitmapFont> = ALL_FONTS
        .iter()
        .map(|f| &***f)
        .filter(|f| {
            // Allow all fonts for small data, progressively restrict for larger
            // The idea: taller fonts take more rows, which affects total image height
            let max_height_for_size = if total_size <= 128 * 1024 {
                20 // Allow any font up to 128 KiB
            } else if total_size <= 512 * 1024 {
                12 // Medium fonts up to 512 KiB
            } else if total_size <= 1024 * 1024 {
                8  // Smaller fonts up to 1 MiB
            } else {
                6  // Only tiny fonts for 1-3 MiB
            };
            f.height <= max_height_for_size
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Use hash to pick from candidates
    let index = (hash as usize) % candidates.len();
    Some(candidates[index])
}
