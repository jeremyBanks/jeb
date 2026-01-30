//! Clean public API for zipng (v2).
//!
//! Provides `encode`/`decode` free functions and builder patterns.

use std::cmp::Ordering;

use crate::checksums::crc32;
use crate::png::palettes;
use crate::polyglot::fonts::{self, FontSelection};

/// Encode files into a polyglot PNG+ZIP.
///
/// Uses default settings: lexicographic sort, auto mode, auto palette, auto font.
pub fn encode<I, K, V>(files: I) -> Vec<u8>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    Encoder::new().encode(files)
}

/// Decode a polyglot PNG+ZIP into files.
///
/// Not yet implemented.
pub fn decode(data: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    Decoder::new().decode(data)
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Color encoding mode.
pub enum EncoderMode {
    /// Indexed for ≤2 MiB, RGBA otherwise.
    Auto,
    /// Force indexed color (8-bit palette).
    Indexed,
    /// Force RGBA (32-bit).
    Rgba,
}

/// Source format hint for decoding.
pub enum Source {
    /// Decode from the ZIP layer.
    Zip,
    /// Decode from the PNG layer.
    Png,
}

/// Named font choice.
pub enum FontChoice {
    Swiss,
    Sixth,
    Sky,
    Monte,
    Sugimori,
    Mini,
    Micro,
}

/// Palette selection strategy.
pub enum PaletteChoice {
    /// Hash-selected from ALL_PALETTES (current behavior).
    Auto,
    /// A specific built-in palette by name.
    Named(NamedPalette),
    /// Raw 768-byte RGB palette.
    Custom(Vec<u8>),
    /// User-provided RGB colors → deduplicate + sort into a 256-entry palette.
    Colors(Vec<[u8; 3]>),
}

/// Named built-in palette (placeholder — add variants as needed).
pub enum NamedPalette {
    // Future: Viridis, Magma, Inferno, etc.
}

// ---------------------------------------------------------------------------
// Encoder
// ---------------------------------------------------------------------------

type CustomCmp = Box<dyn FnMut(&(Vec<u8>, Vec<u8>), &(Vec<u8>, Vec<u8>)) -> Ordering>;

/// Builder for encoding files into a polyglot PNG+ZIP.
pub struct Encoder {
    mode: EncoderMode,
    font: Option<FontChoice>,
    palette: PaletteChoice,
    sorted: bool,
    cmp: Option<CustomCmp>,
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            mode: EncoderMode::Auto,
            font: None,
            palette: PaletteChoice::Auto,
            sorted: true,
            cmp: None,
        }
    }

    pub fn with_mode(mut self, mode: EncoderMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_font(mut self, font: FontChoice) -> Self {
        self.font = Some(font);
        self
    }

    pub fn with_palette(mut self, palette: PaletteChoice) -> Self {
        self.palette = palette;
        self
    }

    pub fn with_sorted(mut self, sorted: bool) -> Self {
        self.sorted = sorted;
        if !sorted {
            self.cmp = None;
        }
        self
    }

    pub fn with_cmp(
        mut self,
        cmp: impl FnMut(&(Vec<u8>, Vec<u8>), &(Vec<u8>, Vec<u8>)) -> Ordering + 'static,
    ) -> Self {
        self.sorted = true;
        self.cmp = Some(Box::new(cmp));
        self
    }

    pub fn encode<I, K, V>(mut self, files: I) -> Vec<u8>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        // 1. Collect files
        let mut collected: Vec<(Vec<u8>, Vec<u8>)> = files
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_vec(), v.as_ref().to_vec()))
            .collect();

        // 2. Sort
        if self.sorted {
            if let Some(ref mut cmp_fn) = self.cmp {
                collected.sort_by(|a, b| cmp_fn(a, b));
            } else {
                collected.sort_by(|a, b| a.0.cmp(&b.0));
            }
        }

        // 3. Compute total size and content hash
        let total_size: usize = collected.iter().map(|(k, v)| k.len() + v.len()).sum();
        let hash = {
            let mut hash_input: Vec<u8> = Vec::new();
            for (path, content) in &collected {
                hash_input.extend_from_slice(path);
                hash_input.push(0);
                hash_input.extend_from_slice(content);
                hash_input.push(0);
            }
            crc32(&hash_input)
        };

        // 4. Determine color mode
        const RGBA_THRESHOLD: usize = 2 * 1024 * 1024;
        let use_rgba = match self.mode {
            EncoderMode::Auto => total_size > RGBA_THRESHOLD,
            EncoderMode::Indexed => false,
            EncoderMode::Rgba => true,
        };

        // 5. Build file slice references
        let file_refs: Vec<(&[u8], &[u8])> = collected
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();

        if use_rgba {
            crate::polyglot::build_polyglot(
                &file_refs,
                0,
                crate::png::BitDepth::EightBit,
                crate::png::ColorType::RedGreenBlueAlpha,
                None,
            )
        } else {
            // 6. Determine palette
            let palette_bytes: Vec<u8> = match self.palette {
                PaletteChoice::Auto => {
                    let palette_index = (hash as usize) % super::ALL_PALETTES.len();
                    let raw = super::ALL_PALETTES[palette_index];
                    palettes::perceptual::deduplicate_rgb_palette(raw)
                }
                PaletteChoice::Named(ref _named) => {
                    // Future: map NamedPalette variants to palette bytes
                    let palette_index = (hash as usize) % super::ALL_PALETTES.len();
                    let raw = super::ALL_PALETTES[palette_index];
                    palettes::perceptual::deduplicate_rgb_palette(raw)
                }
                PaletteChoice::Custom(ref raw) => {
                    palettes::perceptual::deduplicate_rgb_palette(raw)
                }
                PaletteChoice::Colors(ref colors) => {
                    let rgb_colors: Vec<rgb::RGB8> = colors
                        .iter()
                        .map(|c| rgb::RGB8::new(c[0], c[1], c[2]))
                        .collect();
                    let sorted = palettes::perceptual::sort_colors(&rgb_colors);
                    palettes::perceptual::generate(&sorted)
                }
            };

            // 7. Determine font
            let font_override = self.resolve_font(total_size, hash);

            crate::polyglot::build_polyglot_with_font(
                &file_refs,
                0,
                crate::png::BitDepth::EightBit,
                crate::png::ColorType::Indexed,
                Some(&palette_bytes),
                font_override,
            )
        }
    }

    fn resolve_font(&self, total_size: usize, _hash: u32) -> Option<FontSelection> {
        if total_size > fonts::MAX_LABEL_THRESHOLD {
            return None;
        }

        if let Some(ref choice) = self.font {
            let font = match choice {
                FontChoice::Swiss => &*fonts::SWISS,
                FontChoice::Sixth => &*fonts::SIXTH,
                FontChoice::Sky => &*fonts::SKY,
                FontChoice::Monte => &*fonts::MONTE,
                FontChoice::Sugimori => &*fonts::SUGIMORI,
                FontChoice::Mini => &*fonts::MINI,
                FontChoice::Micro => &*fonts::MICRO,
            };
            return Some(FontSelection {
                name_font: font,
                size_font: font,
            });
        }

        // Default v2 font logic: always SWISS for ≤512 KiB
        if total_size <= 512 * 1024 {
            Some(FontSelection {
                name_font: &*fonts::SWISS,
                size_font: &*fonts::SUGIMORI,
            })
        } else if total_size <= 1024 * 1024 {
            Some(FontSelection {
                name_font: &*fonts::MINI,
                size_font: &*fonts::MINI,
            })
        } else {
            Some(FontSelection {
                name_font: &*fonts::MICRO,
                size_font: &*fonts::MICRO,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Decoder (stub)
// ---------------------------------------------------------------------------

/// Builder for decoding a polyglot PNG+ZIP.
pub struct Decoder {
    source: Option<Source>,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self { source: None }
    }

    pub fn with_source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    pub fn decode(self, _data: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        unimplemented!("v2 decode is not yet implemented")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_produces_nonempty_output() {
        let files = vec![
            ("hello.txt", "Hello, world!"),
            ("foo/bar.txt", "Some content"),
            ("README.md", "# Test"),
        ];
        let output = encode(files);
        assert!(!output.is_empty(), "encoded output should not be empty");
        // Should start with PNG signature
        assert_eq!(&output[..4], b"\x89PNG", "output should start with PNG signature");
    }

    #[test]
    fn encoder_builder_works() {
        let output = Encoder::new()
            .with_mode(EncoderMode::Indexed)
            .with_sorted(true)
            .encode(vec![("a.txt", "aaa"), ("b.txt", "bbb")]);
        assert!(!output.is_empty());
        assert_eq!(&output[..4], b"\x89PNG");
    }

    #[test]
    fn unsorted_preserves_order() {
        // Just verify it doesn't panic
        let output = Encoder::new()
            .with_sorted(false)
            .encode(vec![("z.txt", "zzz"), ("a.txt", "aaa")]);
        assert!(!output.is_empty());
    }
}
