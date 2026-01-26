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

// PNG module with palettes
pub mod png;

// Text/fonts module
pub mod text;

// ZIP module
pub mod zip;

// ZipNG module (renamed from zipng to avoid crate name collision)
#[path = "zipng_impl.rs"]
pub mod zipng_core;

// Polyglot module (our main work)
pub mod polyglot;

#[cfg(feature = "brotli")]
pub mod brotli;

// Legacy modules
pub mod font;
pub mod padding;

#[cfg(feature = "dev-dependencies")]
pub mod dev;

// Re-export key types
pub use crate::png::{BitDepth, ColorType};
pub use crate::png::palettes;

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
