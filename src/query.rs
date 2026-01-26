mod text_to_speech;

use std::fmt::Debug;

use async_trait::async_trait;
use derive_more::From;
use derive_more::TryInto;
use miette::Diagnostic;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub use self::text_to_speech::TextToSpeech;
pub use self::text_to_speech::TextToSpeechResponse;
use crate::backend::BackendError;
use crate::blobs::bytes;
use crate::blobs::Postcard;
use crate::never;
use crate::panic;
use crate::Blip;
use crate::Blob;
use crate::Blobbable;
use crate::Context;
#[cfg(doc)]
use crate::*;

/// An enum that may contain any [`Request`] type.
#[derive(Debug, Serialize, Deserialize, Clone, TryInto, From)]
#[repr(u32)]
pub enum AnyRequest {
    TextToSpeech(TextToSpeech) = TextToSpeech::TAG,
}

/// An enum that may contain any [`Response`] type.
#[derive(Debug, Serialize, Deserialize, Clone, TryInto, From)]
#[repr(u32)]
pub enum AnyResponse {
    TextToSpeech(TextToSpeechResponse) = TextToSpeech::TAG,
}

#[async_trait]
/// The input for an operation that can be executed by the [`Engine`].
pub trait Request: Default + Debug + Blobbable + Clone + Sync + Send {
    const TAG: u32;
    type Response: self::Response;

    async fn execute(&self, context: &mut Context) -> Result<Self::Response, BackendError> {
        Err(BackendError::NotSupported)?
    }
}

///  The output of an operation executed by the [`Engine`].
pub trait Response: Default + Debug + Blobbable + Clone + Sync + Send {
    type Request: self::Request;
}
