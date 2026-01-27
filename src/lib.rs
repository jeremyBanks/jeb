#![feature(doc_cfg)]
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

// ZipNG module (renamed from zipng to avoid crate name collision)
pub mod zipng_impl;

// Polyglot module (our main work)
pub mod polyglot;

#[cfg(feature = "brotli")]
pub mod brotli;

// Legacy modules
pub mod font;
pub mod padding;

#[cfg(feature = "dev-dependencies")]
pub mod dev;

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
    pub use crate::padding;
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

/// Creates a polyglot PNG+ZIP file
pub fn zipng(files: &Files) -> Vec<u8> {
    let file_list: Vec<(&[u8], &[u8])> = files
        .files
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();

    polyglot::build_polyglot(
        &file_list,
        0,  // auto width
        crate::png::BitDepth::EightBit,
        crate::png::ColorType::Indexed,
        None,
    )
}

/// Creates a simple zip file
pub fn zip(files: &Files) -> Vec<u8> {
    crate::zip::zip(files.files.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
}
