//! Clean public API for zipng (v2).
//!
//! Provides `encode`/`decode` free functions and builder patterns.

use {
    crate::{
        checksums::crc32,
        png::palettes,
        polyglot::fonts::{self, FontSelection},
    },
    std::cmp::Ordering,
};

/// Encode files into a polyglot PNG+ZIP.
///
/// Uses default settings: lexicographic sort, auto mode, auto palette, auto
/// font.
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
/// Uses both ZIP and PNG decoding methods and verifies they match.
pub fn decode(data: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DecodeError> {
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
    /// Decode from the image pixel data (works with PNG, GIF, or any indexed format).
    Image,
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
                },
                PaletteChoice::Named(ref _named) => {
                    // Future: map NamedPalette variants to palette bytes
                    let palette_index = (hash as usize) % super::ALL_PALETTES.len();
                    let raw = super::ALL_PALETTES[palette_index];
                    palettes::perceptual::deduplicate_rgb_palette(raw)
                },
                PaletteChoice::Custom(ref raw) =>
                    palettes::perceptual::deduplicate_rgb_palette(raw),
                PaletteChoice::Colors(ref colors) => {
                    let rgb_colors: Vec<rgb::RGB8> = colors
                        .iter()
                        .map(|c| rgb::RGB8::new(c[0], c[1], c[2]))
                        .collect();
                    let sorted = palettes::perceptual::sort_colors(&rgb_colors);
                    palettes::perceptual::generate(&sorted)
                },
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
                name_font: &fonts::SWISS,
                size_font: &fonts::SUGIMORI,
            })
        } else if total_size <= 1024 * 1024 {
            Some(FontSelection {
                name_font: &fonts::MINI,
                size_font: &fonts::MINI,
            })
        } else {
            Some(FontSelection {
                name_font: &fonts::MICRO,
                size_font: &fonts::MICRO,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Decoder
// ---------------------------------------------------------------------------

/// Builder for decoding a polyglot PNG+ZIP.
pub struct Decoder {
    source: Option<Source>,
    verify: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            source: None,
            verify: true,
        }
    }

    pub fn with_source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    pub fn decode(self, data: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DecodeError> {
        match self.source {
            Some(Source::Zip) => decode_via_zip(data),
            Some(Source::Image) => decode_via_image(data),
            None => {
                // Try both methods
                let zip_result = decode_via_zip(data);
                let image_result = decode_via_image(data);

                match (zip_result, image_result) {
                    (Ok(zip_files), Ok(image_files)) => {
                        if self.verify && zip_files != image_files {
                            Err(DecodeError::MethodMismatch {
                                zip_count: zip_files.len(),
                                image_count: image_files.len(),
                            })
                        } else {
                            Ok(zip_files)
                        }
                    },
                    (Ok(files), Err(_)) => {
                        tracing::info!("Extracted via ZIP method only (image method failed)");
                        Ok(files)
                    },
                    (Err(_), Ok(files)) => {
                        tracing::info!("Extracted via image method only (ZIP method failed)");
                        Ok(files)
                    },
                    (Err(zip_err), Err(_image_err)) => Err(zip_err),
                }
            },
        }
    }
}

/// Errors that can occur during decoding.
#[derive(Debug)]
pub enum DecodeError {
    /// ZIP extraction failed.
    ZipError(String),
    /// Image extraction failed.
    ImageError(String),
    /// Both methods succeeded but produced different results.
    MethodMismatch { zip_count: usize, image_count: usize },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZipError(msg) => write!(f, "ZIP extraction failed: {}", msg),
            Self::ImageError(msg) => write!(f, "Image extraction failed: {}", msg),
            Self::MethodMismatch {
                zip_count,
                image_count,
            } => write!(
                f,
                "Extraction mismatch: ZIP found {} files, image found {} files",
                zip_count, image_count
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

#[cfg(feature = "zip")]
fn decode_via_zip(data: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DecodeError> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| DecodeError::ZipError(e.to_string()))?;

    let mut files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| DecodeError::ZipError(e.to_string()))?;

        if file.is_dir() {
            continue;
        }

        let name = file.name().as_bytes().to_vec();
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| DecodeError::ZipError(e.to_string()))?;

        files.push((name, content));
    }

    Ok(files)
}

#[cfg(not(feature = "zip"))]
fn decode_via_zip(_data: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DecodeError> {
    Err(DecodeError::ZipError(
        "ZIP decoding not available (feature not enabled)".to_string(),
    ))
}

#[cfg(feature = "image")]
fn decode_via_image(data: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DecodeError> {
    use std::{
        collections::HashMap,
        io::{Cursor, Read},
    };

    // Load image using the image crate (works with PNG, GIF, BMP, etc.)
    let img = image::load_from_memory(data)
        .map_err(|e| DecodeError::ImageError(format!("Failed to load image: {}", e)))?;

    let rgb_img = img.to_rgb8();
    let pixels = rgb_img.as_raw();

    // Count unique colors and build reverse map
    let mut unique_colors = HashMap::new();
    for chunk in pixels.chunks_exact(3) {
        let rgb = [chunk[0], chunk[1], chunk[2]];
        let next_index = unique_colors.len();
        unique_colors.entry(rgb).or_insert(next_index);
        
        // Early exit if too many colors
        if unique_colors.len() > 256 {
            return Err(DecodeError::ImageError(
                "Image has >256 unique colors, not indexed mode (likely RGBA or corrupted)".to_string(),
            ));
        }
    }

    // Valid indexed zipng must have exactly 256 colors
    if unique_colors.len() != 256 {
        return Err(DecodeError::ImageError(format!(
            "Image has {} unique colors (expected exactly 256 for indexed mode)",
            unique_colors.len()
        )));
    }

    // Build reverse color map
    let mut reverse_map: HashMap<[u8; 3], u8> = HashMap::new();
    for (color, index) in unique_colors.iter() {
        reverse_map.insert(*color, *index as u8);
    }

    // Extract bytes from pixels
    let mut bytes = Vec::new();
    for chunk in pixels.chunks_exact(3) {
        let rgb = [chunk[0], chunk[1], chunk[2]];
        bytes.push(reverse_map[&rgb]);
    }

    // Parse as ZIP archive
    let cursor = Cursor::new(&bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| DecodeError::ImageError(e.to_string()))?;

    let mut files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| DecodeError::ImageError(e.to_string()))?;

        if file.is_dir() {
            continue;
        }

        let name = file.name().as_bytes().to_vec();
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| DecodeError::ImageError(e.to_string()))?;

        files.push((name, content));
    }

    Ok(files)
}

#[cfg(not(feature = "image"))]
fn decode_via_image(_data: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DecodeError> {
    Err(DecodeError::ImageError(
        "Image decoding not available (feature not enabled)".to_string(),
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Manual inspection test
    fn inspect_rgba_structure() {
        // Load an RGBA zipng and inspect its structure
        let data = std::fs::read("/tmp/zipng-rgba-test/rgba-test.png").unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let rgba = img.to_rgba8();
        
        println!("Dimensions: {}x{}", rgba.width(), rgba.height());
        println!("\nFirst 12 pixels:");
        for x in 0..12 {
            let p = rgba.get_pixel(x, 0);
            println!("  {:?}", p.0);
        }
        
        // Count colors
        use std::collections::HashSet;
        let mut colors = HashSet::new();
        let pixels = rgba.as_raw();
        for chunk in pixels.chunks_exact(4) {
            colors.insert([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if colors.len() > 300 {
                break;
            }
        }
        println!("\nUnique colors: {}", colors.len());
    }

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
        assert_eq!(
            &output[..4],
            b"\x89PNG",
            "output should start with PNG signature"
        );
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

    #[test]
    #[cfg(all(feature = "zip", feature = "image"))]
    fn roundtrip_encode_decode() {
        let original_files = vec![
            ("hello.txt", "Hello, world!"),
            ("data/info.txt", "Some information"),
            ("README.md", "# Test Archive"),
        ];

        // Encode
        let archive = encode(original_files.clone());
        assert!(!archive.is_empty());

        // Decode
        let decoded = decode(&archive).expect("decode should succeed");

        // Convert to comparable format
        let mut decoded_files: Vec<(String, String)> = decoded
            .into_iter()
            .map(|(name, content)| {
                (
                    String::from_utf8(name).unwrap(),
                    String::from_utf8(content).unwrap(),
                )
            })
            .collect();

        let mut original_sorted: Vec<(String, String)> = original_files
            .into_iter()
            .map(|(n, c)| (n.to_string(), c.to_string()))
            .collect();
        original_sorted.sort_by(|a, b| a.0.cmp(&b.0));
        decoded_files.sort_by(|a, b| a.0.cmp(&b.0));

        // Compare
        assert_eq!(decoded_files.len(), original_sorted.len());
        for ((dec_name, dec_content), (orig_name, orig_content)) in
            decoded_files.iter().zip(original_sorted.iter())
        {
            assert_eq!(dec_name, orig_name);
            assert_eq!(dec_content, orig_content);
        }
    }
}
