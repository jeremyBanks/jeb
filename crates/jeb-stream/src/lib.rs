#![allow(clippy::type_complexity)]
#![allow(dead_code)]
mod item;
pub mod split;
pub mod stream_utils;
mod streams;
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
pub trait StreamExt: futures::stream::Stream + futures::stream::StreamExt {}
impl<T> StreamExt for T where T: futures::stream::StreamExt {}
pub trait TryStreamExt:
    StreamExt
    + futures::stream::Stream
    + futures::stream::StreamExt
    + futures::stream::TryStream
    + futures::stream::TryStreamExt
{
}
impl<T> TryStreamExt for T where T: futures::stream::TryStreamExt {}
