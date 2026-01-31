#![doc = include_str!("../README.md")]
#![allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_crate_dependencies,
    unused_imports,
    missing_docs,
)]

use indexmap::IndexMap;

// Core modules - make them public for the impl module
pub mod checksums;
pub mod deflate;
pub mod generic;
pub mod io;
pub mod opstructs;
pub mod zlib;

// Re-export commonly used items at crate root for internal use
pub use checksums::{adler32, crc32};
pub use generic::{default, never, panic, PhantomType};
pub use io::{OutputBuffer, output_buffer, Offset};
pub use deflate::{write_deflate, DeflateMode};
pub use zlib::write_zlib;

// Constants
pub const PNG_HEADER_SIZE: usize = 33;  // 8 (sig) + 25 (IHDR chunk)

// PNG module with palettes
pub mod png;

// Text/fonts module
pub mod text;

// ZIP module
pub mod zip;

// Polyglot module (our main work)
pub mod polyglot;

pub mod v2;

#[cfg(feature = "brotli")]
pub mod brotli;

// Legacy modules
pub mod font;


// Re-export key types from png
pub use crate::png::{BitDepth, ColorType, Png, ToPng};
pub use crate::png::palettes;
pub use crate::png::sizes::{PNG_CHUNK_PREFIX_SIZE, PNG_CHUNK_SUFFIX_SIZE, PNG_CHUNK_WRAPPER_SIZE};
pub use crate::png::write_png;
// Re-export ColorType variants for convenience
pub use crate::png::ColorType::{Luminance, LuminanceAlpha, RedGreenBlue, RedGreenBlueAlpha, Indexed};
// Re-export BitDepth variants for convenience
pub use crate::png::BitDepth::{OneBit, TwoBit, FourBit, EightBit, SixteenBit};

// Re-export key types from zip
pub use crate::zip::{Zip, ToZip, ZipConfiguration, ZipEntry, ZipEntryComparison};

// Re-export Font from font module
pub use crate::font::Font;

/// Unstable implementation module
pub mod r#impl {
    pub use crate::checksums;
    pub use crate::deflate;
    pub use crate::font;
    pub use crate::generic;
    pub use crate::png;
    pub use crate::polyglot;
    pub use crate::text;
    pub use crate::zip;
    pub use crate::zlib;

    #[cfg(feature = "brotli")]
    pub use crate::brotli;
}

/// Files for a zip archive
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Files {
    pub files: IndexMap<Vec<u8>, Vec<u8>>,
}

impl From<IndexMap<Vec<u8>, Vec<u8>>> for Files {
    fn from(files: IndexMap<Vec<u8>, Vec<u8>>) -> Self {
        Files { files }
    }
}

/// Threshold for switching from indexed color to RGBA (2 MiB)
const RGBA_THRESHOLD: usize = 2 * 1024 * 1024;

/// All available palettes for deterministic selection.
/// Only sequential palettes are used, ensuring a smooth monotonic color progression
/// that works well with the cycling palette index pattern in spacing rows.
/// Filtered to only include palettes with good contrast (luminance diff >= 150)
/// between first and last colors, ensuring readable filename labels.
/// Excludes: TURBO (poor label contrast), all diverging, dual-sequential, and cyclic palettes.
static ALL_PALETTES: &[&[u8]] = &[
    palettes::oceanic::AMP, palettes::oceanic::ICE, palettes::oceanic::OXY,
    palettes::crameri::BUDA, palettes::crameri::NUUK, palettes::crameri::OSLO,
    palettes::oceanic::DEEP, palettes::oceanic::RAIN,
    palettes::crameri::ACTON, palettes::crameri::DAVOS, palettes::crameri::DEVON,
    palettes::crameri::IMOLA, palettes::crameri::LAPAZ, palettes::crameri::TOKYO,
    palettes::crameri::TURKU, palettes::oceanic::ALGAE, palettes::oceanic::DENSE,
    palettes::oceanic::SOLAR, palettes::oceanic::SPEED, palettes::oceanic::TEMPO,
    palettes::viridis::MAGMA, palettes::crameri::BAMAKO,
    palettes::crameri::BATLOW, palettes::crameri::BILBAO, palettes::crameri::HAWAII,
    palettes::oceanic::HALINE, palettes::oceanic::MATTER, palettes::oceanic::TURBID,
    palettes::viridis::PLASMA, palettes::crameri::LAJOLLA, palettes::oceanic::THERMAL,
    palettes::singles::CIVIDIS, palettes::viridis::INFERNO, palettes::viridis::VIRIDIS,
    palettes::crameri::BATLOW_K, palettes::crameri::BATLOW_W,
];

/// Creates a polyglot PNG+ZIP file.
///
/// Files are sorted lexicographically by path. The color mode is chosen based on
/// total data size:
/// - For data > 2 MiB: RGBA mode for better density (4 bytes per pixel)
/// - For smaller data: Indexed color with a deterministically-selected palette
///   based on a hash of the input data
pub fn zipng(files: &Files) -> Vec<u8> {
    zipng_with_palette(files, None)
}

/// Like [`zipng`], but allows specifying a custom 768-byte RGB palette.
///
/// If `palette` is `Some`, it is used directly (forcing indexed color mode for
/// data that would fit). If `None`, falls back to the default hash-selected
/// palette.
pub fn zipng_with_palette(files: &Files, palette: Option<&[u8]>) -> Vec<u8> {
    // Sort files lexicographically by path
    let mut sorted_files: Vec<(&[u8], &[u8])> = files
        .files
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();
    sorted_files.sort_by_key(|(a, _)| *a);

    // Calculate total data size
    let total_size: usize = sorted_files.iter().map(|(_, v)| v.len()).sum();

    if total_size > RGBA_THRESHOLD {
        // Large data (>2 MiB): use RGBA for best density (4 bytes/pixel)
        polyglot::build_polyglot(
            &sorted_files,
            0,
            crate::png::BitDepth::EightBit,
            crate::png::ColorType::RedGreenBlueAlpha,
            None,
        )
    } else {
        // Smaller data: use indexed color
        let selected_palette;
        let palette_slice: &[u8] = if let Some(p) = palette {
            p
        } else {
            // Hash all file paths and contents to deterministically select a palette
            let mut hash_input: Vec<u8> = Vec::new();
            for (path, content) in &sorted_files {
                hash_input.extend_from_slice(path);
                hash_input.push(0); // separator
                hash_input.extend_from_slice(content);
                hash_input.push(0); // separator
            }
            let hash = crc32(&hash_input);
            let palette_index = (hash as usize) % ALL_PALETTES.len();
            selected_palette = ALL_PALETTES[palette_index];
            selected_palette
        };

        let deduped = palettes::perceptual::deduplicate_rgb_palette(palette_slice);
        polyglot::build_polyglot(
            &sorted_files,
            0,
            crate::png::BitDepth::EightBit,
            crate::png::ColorType::Indexed,
            Some(&deduped),
        )
    }
}

/// Creates a simple zip file
pub fn zip(files: &Files) -> Vec<u8> {
    crate::zip::zip(files.files.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
}
