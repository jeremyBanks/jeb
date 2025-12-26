#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

mod examples;
mod sink;
mod source;
mod transform;

use derive_more::{Deref, DerefMut};
pub use examples::*;
use jeb_values::Bytes;
pub use sink::*;
pub use source::*;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
pub use transform::*;

use crate::{Receiver, Sender, channel};

pub type TaskHandle = tokio::task::JoinHandle<()>;
