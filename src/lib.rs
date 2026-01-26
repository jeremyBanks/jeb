#![feature(doc_cfg, doc_auto_cfg)]
#![allow(non_upper_case_globals)]
#![doc = include_str!("../README.md")]
//!
//! ## Feature flags
#![doc = document_features!()]

// Relaxed lints for now while merging
#![allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_crate_dependencies,
    unused_imports,
    missing_docs,
)]

use derive_more::From;
use derive_more::Into;
use document_features::document_features;
use indexmap::IndexMap;
use tap::Tap;
use tracing::warn;

#[doc(hidden)]
use crate as zipng;
use crate::generic::default;
use crate::generic::noop_mut;
use crate::png::BitDepth;
use crate::png::ColorMode;

// Core modules (from dev branch's modular structure)
mod checksums;
mod deflate;
mod generic;
mod io;
mod opstructs;
mod zlib;

// PNG module with palettes (from dev)
mod png;

// Text/fonts module
mod text;

// ZIP module
mod zip;

// ZipNG module
mod zipng_impl;

// Polyglot module (from trunk)
pub mod polyglot;

#[cfg(feature = "brotli")]
pub mod brotli;

// Legacy modules kept for compatibility
pub mod font;
pub mod padding;

#[cfg(feature = "dev-dependencies")]
pub mod dev;

// Re-export everything for public API
pub use crate::checksums::*;
pub use crate::deflate::*;
pub use crate::generic::*;
pub use crate::io::*;
pub use crate::opstructs::*;
pub use crate::png::*;
pub use crate::text::*;
pub use crate::zip::*;
pub use crate::zipng_impl::*;
pub use crate::zlib::*;

/// Unstable implementation details module (for backward compat)
pub mod r#impl {
    #![doc(cfg(all(internal, unstable)))]
    #![allow(missing_docs)]

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

/// Creates a zip file.
pub fn zip(files: &Files) -> Vec<u8> {
    zip_with(files, noop_mut)
}

/// Creates a zip file using custom options.
pub fn zip_with(files: &Files, opts: Opts<ZipOptions>) -> Vec<u8> {
    let _opts = ZipOptions::default().tap_mut(opts);
    crate::zip::zip(files.files.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
}

/// Creates a "transparent zipng" zip file with the given files.
pub fn zipng(files: &Files) -> Vec<u8> {
    zipng_with(files, noop_mut)
}

type Opts<Options> = fn(&mut Options);

/// Creates a "transparent zipng" polyglot PNG+ZIP file.
pub fn zipng_with(files: &Files, opts: Opts<ZipngOptions>) -> Vec<u8> {
    let file_list: Vec<(&[u8], &[u8])> = files
        .files
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();

    let total_size: usize = file_list
        .iter()
        .map(|(name, body)| 30 + name.len() + body.len())
        .sum();

    let mut opts = ZipngOptions::default_for_data(&vec![0u8; total_size]).tap_mut(opts);

    if opts.png.width == 0 {
        opts.png.width = 64;
    }

    polyglot::build_polyglot(
        &file_list,
        opts.png.width as u32,
        opts.png.bit_depth,
        opts.png.color_mode,
        opts.png.color_palette.as_deref(),
    )
}

/// Creates a zip file with uncompressed files.
pub fn sliceable_zip(files: &Files) -> Vec<u8> {
    sliceable_zip_with(files, noop_mut)
}

/// Creates a zip file with uncompressed files using custom options.
pub fn sliceable_zip_with(files: &Files, opts: Opts<ZipOptions>) -> Vec<u8> {
    let _opts = ZipOptions::default().tap_mut(opts);
    todo!()
}

/// Creates a PNG file.
pub fn png(body: &[u8]) -> Vec<u8> {
    png_with(body, noop_mut)
}

/// Creates a PNG file with options.
pub fn png_with(body: &[u8], opts: Opts<PngOptions>) -> Vec<u8> {
    let mut opts = PngOptions::default().tap_mut(opts);

    if opts.width == 0 {
        opts.width = 64;
    }

    let bytes_per_pixel =
        (opts.bit_depth.bits_per_sample() * opts.color_mode.samples_per_pixel() + 7) / 8;
    let bytes_per_row = opts.width * bytes_per_pixel;
    let height = (body.len() + bytes_per_row - 1) / bytes_per_row;

    let mut buffer = Vec::new();
    crate::png::write_png(
        &mut buffer,
        body,
        opts.width as u32,
        height as u32,
        opts.png_bit_depth(),
        opts.png_color_mode(),
        opts.color_palette.as_deref(),
    );
    buffer
}

#[cfg(feature = "brotli")]
/// Creates a brotli-compressed polyglot.
pub fn zipngbr(files: &Files) -> Vec<u8> {
    zipngbr_with(files, noop_mut)
}

#[cfg(feature = "brotli")]
/// Creates a brotli-compressed polyglot with options.
pub fn zipngbr_with(files: &Files, opts: Opts<ZipngBrOptions>) -> Vec<u8> {
    let _opts = ZipngBrOptions::default().tap_mut(opts);
    brotli::compress(zipng(files).as_slice()).to_vec()
}

/// Files for a zip archive.
#[derive(Debug, Default, Clone, PartialEq, Eq, From, Into)]
pub struct Files {
    pub files: IndexMap<Vec<u8>, Vec<u8>>,
}

/// Zip archive options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct ZipOptions {}

/// PNG encoding options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct PngOptions {
    pub width: usize,
    pub max_height: usize,
    pub bit_depth: BitDepth,
    pub color_mode: ColorMode,
    pub color_palette: Option<Vec<u8>>,
}

impl PngOptions {
    fn png_bit_depth(&self) -> crate::png::BitDepth {
        self.bit_depth
    }

    fn png_color_mode(&self) -> crate::png::ColorMode {
        self.color_mode
    }
}

#[cfg(feature = "brotli")]
/// Brotli compression options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct BrotliOptions {}

/// Combined PNG+ZIP options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, From)]
#[non_exhaustive]
pub struct ZipngOptions {
    pub png: PngOptions,
    pub zip: ZipOptions,
}

impl ZipngOptions {
    pub fn default_for_data(data: &[u8]) -> Self {
        use crate::png::{EightBit, OneBit, TwoBit, Indexed, Lightness, RedGreenBlue, RedGreenBlueAlpha};

        let mut opts = Self::default();
        opts.png.bit_depth = EightBit;
        opts.png.color_mode = Indexed;
        opts.png.max_height = 8192;

        match data.len() {
            len @ 0x0..=0x20 => {
                opts.png.bit_depth = OneBit;
                opts.png.color_mode = Lightness;
                opts.png.width = 16.min(len * 8);
            },
            0x21..=0x100 => {
                opts.png.bit_depth = TwoBit;
                opts.png.color_mode = Lightness;
                opts.png.width = 16;
            },
            0x101..=0x200 => opts.png.width = 16,
            0x201..=0x800 => opts.png.width = 32,
            0x801..=0x2000 => opts.png.width = 64,
            0x2001..=0x8000 => opts.png.width = 128,
            0x8001..=0x20000 => opts.png.width = 256,
            0x20001..=0x80000 => opts.png.width = 512,
            0x80001..=0x200000 => opts.png.width = 1024,
            0x200001..=0x800000 => {
                opts.png.width = 1024;
                opts.png.color_mode = RedGreenBlue;
            },
            len => {
                opts.png.width = 1024;
                opts.png.color_mode = RedGreenBlueAlpha;
                warn!("zip data size is too large ({len} bytes)");
            },
        }

        opts
    }
}

#[cfg(feature = "brotli")]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, From)]
#[non_exhaustive]
pub struct ZipngBrOptions {
    pub png: PngOptions,
    pub zip: ZipOptions,
    pub br: BrotliOptions,
}
