#![feature(doc_cfg, doc_auto_cfg)]
#![allow(non_upper_case_globals)]
#![warn(unused_crate_dependencies, missing_docs, clippy::redundant_pub_crate)]
#![cfg_attr(
    all(debug_assertions, any(not(test), feature = "EDITOR")),
    allow(
        dead_code,
        unreachable_code,
        unused_variables,
        unused_crate_dependencies,
        unused_imports,
        missing_docs,
    )
)]
#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

use crate::generic::*;

mod checksums;
mod deflate;
mod generic;
mod io;
mod png;
mod text;
mod zip;
mod zipng;
mod zlib;

#[cfg(feature = "dev-dependencies")]
pub mod dev;
mod opstructs;

#[doc(hidden)]
pub use crate::text::*;
#[doc(inline)]
pub use crate::{
    checksums::*, deflate::*, generic::*, io::*, opstructs::*, png::*, zip::*, zipng::*, zipng::*,
    zlib::*,
};
