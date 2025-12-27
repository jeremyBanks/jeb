#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

mod examples;
mod sink;
mod source;
mod transform;

use {
    crate::{
        Receiver,
        Sender,
        channel,
    },
    derive_more::{
        Deref,
        DerefMut,
    },
    jeb_value::Bytes,
    tokio::io::{
        AsyncRead,
        AsyncReadExt,
        AsyncWrite,
        AsyncWriteExt,
    },
};
pub use {
    examples::*,
    sink::*,
    source::*,
    transform::*,
};

pub type TaskHandle = tokio::task::JoinHandle<()>;
