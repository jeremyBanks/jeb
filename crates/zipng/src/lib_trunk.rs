#![feature(doc_cfg)]
#![doc = include_str!("../README.md")]
//!
//! ## Feature flags
#![doc = document_features!()]

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
use crate::png::EightBit;
use crate::png::FourBit;
use crate::png::Indexed;
use crate::png::Lightness;
use crate::png::OneBit;
use crate::png::RedGreenBlue;
use crate::png::RedGreenBlueAlpha;
use crate::png::TwoBit;
use crate::png::PALLETTE_8_BIT_DATA;
#[doc(hidden)]
pub use crate::zipng::r#impl::*;

pub mod r#impl {
    //! Unstable implementation details. For entertainment use only.
    //!
    //! This isn't actually behind a feature gate, but you'll need to import it
    //! as `zipng::r#impl` because `impl` is a keyword.
    #![doc(cfg(all(internal, unstable)))]
    #![path = "."]
    #![allow(missing_docs)]

    #[cfg(feature = "brotli")]
    pub mod brotli;

    pub mod checksums;
    pub mod deflate;
    pub mod font;
    pub mod generic;
    pub mod padding;
    pub mod png;
    pub mod polyglot;
    pub mod text;
    pub mod zip;
    pub mod zlib;
}

/// Creates a zip file.
pub fn zip(files: &Files) -> Vec<u8> {
    zip_with(files, noop_mut)
}

/// Creates a zip file using custom options.
pub fn zip_with(files: &Files, opts: Opts<ZipOptions>) -> Vec<u8> {
    let _opts = ZipOptions::default().tap_mut(opts);
    zip::zip(files.files.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
}

/// Creates a "transparent zipng" zip file with the given files, in the given
/// order.
pub fn zipng(files: &Files) -> Vec<u8> {
    zipng_with(files, noop_mut)
}

type Opts<Options> = fn(&mut Options);

/// Creates a "transparent zipng" zip file using custom options, with the
/// given files, in the given order.
///
/// The resulting file is simultaneously a valid PNG image and a valid ZIP archive.
/// PNG readers will display the ZIP data as pixels, while ZIP readers will
/// extract the embedded files.
pub fn zipng_with(files: &Files, opts: Opts<ZipngOptions>) -> Vec<u8> {
    // Build the file list for the polyglot builder
    let file_list: Vec<(&[u8], &[u8])> = files
        .files
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();

    // Estimate total data size for default options
    let total_size: usize = file_list
        .iter()
        .map(|(name, body)| 30 + name.len() + body.len())
        .sum();

    let mut opts = ZipngOptions::default_for_data(&vec![0u8; total_size]).tap_mut(opts);

    // Ensure reasonable defaults
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
/// Creates a zip file wherein all files are stored un-compressed, directly in
/// the zip file as-is.
pub fn sliceable_zip(files: &Files) -> Vec<u8> {
    sliceable_zip_with(files, noop_mut)
}

/// Creates a zip file using custom options wherein all files are stored
/// un-compressed, directly in the zip file as-is.
pub fn sliceable_zip_with(files: &Files, opts: Opts<ZipOptions>) -> Vec<u8> {
    let opts = ZipOptions::default().tap_mut(opts);
    todo!()
}

/// Creates a PNG file with the given image data.
///
/// If you have a real image, you probably want to use [`png_with`] instead
/// so you can specify the actual image dimensions and color information,
/// instead of using this function's arbitrary choices.
pub fn png(body: &[u8]) -> Vec<u8> {
    png_with(body, noop_mut)
}

/// Creates a PNG file with the given image data and options.
pub fn png_with(body: &[u8], opts: Opts<PngOptions>) -> Vec<u8> {
    let mut opts = PngOptions::default().tap_mut(opts);

    // Calculate dimensions if not specified
    if opts.width == 0 {
        opts.width = 64;
    }

    let bytes_per_pixel =
        (opts.bit_depth.bits_per_sample() * opts.color_mode.samples_per_pixel() + 7) / 8;
    let bytes_per_row = opts.width * bytes_per_pixel;
    let height = (body.len() + bytes_per_row - 1) / bytes_per_row;

    let mut buffer = Vec::new();
    png::write_png(
        &mut buffer,
        body,
        opts.width as u32,
        height as u32,
        opts.bit_depth,
        opts.color_mode,
        opts.color_palette.as_deref(),
    );
    buffer
}

#[cfg(feature = "brotli")]
/// Creates a "transparent zipng" zip file with the given files, in the given
/// order, and then compresses it with `brotli`.
pub fn zipngbr(files: &Files) -> Vec<u8> {
    zipngbr_with(files, noop_mut)
}

#[cfg(feature = "brotli")]
/// Creates a "transparent zipng" zip file using custom options, with the
/// given files, in the given order, and then compresses it with `brotli`.
pub fn zipngbr_with(files: &Files, opts: Opts<ZipngBrOptions>) -> Vec<u8> {
    let _opts = ZipngBrOptions::default().tap_mut(opts);
    brotli::compress(zipng(files).as_slice()).to_vec()
}

/// Files to be included in a zip archive.
#[derive(Debug, Default, Clone, PartialEq, Eq, From, Into)]
pub struct Files {
    files: IndexMap<Vec<u8>, Vec<u8>>,
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

#[cfg(feature = "brotli")]
/// Brotli compression options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct BrotliOptions {}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, From)]
#[non_exhaustive]
pub struct ZipngOptions {
    pub png: PngOptions,
    pub zip: ZipOptions,
}

impl ZipngOptions {
    pub fn default_for_data(data: &[u8]) -> Self {
        let mut opts = Self::default();

        opts.png.bit_depth = EightBit;
        opts.png.color_mode = Indexed;
        opts.png.color_palette = Some(PALLETTE_8_BIT_DATA.to_vec());
        opts.png.max_height = 8192;

        match data.len() {
            len @ 0x0..=0x20 => {
                opts.png.color_palette = None;
                opts.png.bit_depth = OneBit;
                opts.png.color_mode = Lightness;
                opts.png.width = 16.min(len * 8);
            },
            0x21..=0x100 => {
                opts.png.color_palette = None;
                opts.png.bit_depth = TwoBit;
                opts.png.color_mode = Lightness;
                opts.png.width = 16;
            },
            0x101..=0x200 => {
                opts.png.width = 16;
            },
            0x201..=0x800 => {
                opts.png.width = 32;
            },
            0x801..=0x2000 => {
                opts.png.width = 64;
            },
            0x2001..=0x8000 => {
                opts.png.width = 128;
            },
            0x8001..=0x20000 => {
                opts.png.width = 256;
            },
            0x20001..=0x80000 => {
                opts.png.width = 512;
            },
            0x80001..=0x200000 => {
                opts.png.width = 1024;
            },
            0x200001.. => {
                opts.png.width = 1024;
                opts.png.color_palette = None;
                opts.png.color_mode = RedGreenBlueAlpha;
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
