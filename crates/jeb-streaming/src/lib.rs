#![allow(clippy::type_complexity)]
#![allow(dead_code)]

mod item;

// Stream infrastructure
pub mod split;
pub mod stream_utils;

// Stream functions (promoted from streams module)
mod streams;

// Re-export everything
pub use {
    item::*,
    split::{
        ErrStream,
        OkStream,
        oks_and_errs,
    },
    stream_utils::{
        errs,
        fail_fast,
        oks,
        unwrap_oks,
    },
    streams::*,
};
