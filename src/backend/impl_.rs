use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use miette::Diagnostic;
use once_cell::sync::Lazy;
use static_assertions::assert_impl_all;
use static_assertions::assert_obj_safe;
use thiserror::Error;
use tracing::info;

use super::error::BackendError;
use crate::backends::sqlite::SqliteStorage;
use crate::blobs::blip;
use crate::blobs::blip::blip;
use crate::blobs::blob;
use crate::blobs::Blip;
use crate::blobs::Blob;
use crate::blobs::BlobSerialization;
use crate::blobs::UnknownBlip;
use crate::blobs::UnknownBlob;
use crate::never;
use crate::panic;
use crate::query;
use crate::Blobbable;
use crate::Metadata;
use crate::Request;

// assert_impl_all!(Backend: BackendImpl);
// assert_impl_all!(Backend: BackendExt);
assert_obj_safe!(BackendImpl);

/// Internal minimally-typed object-safe implementation of an [`Incremental`]
/// backend.
#[async_trait]
pub trait BackendImpl: Debug + Send + Sync {
    async fn insert_blob_impl(&self, blob: &UnknownBlob) -> Result<UnknownBlip, BackendError> {
        Err(BackendError::NotSupported)
    }

    async fn get_blob_impl(&self, blip: UnknownBlip) -> Result<Option<UnknownBlob>, BackendError> {
        Err(BackendError::NotSupported)
    }

    async fn insert_response_impl(
        &self,
        request: UnknownBlip,
        response: UnknownBlip,
    ) -> Result<UnknownResponseItem, BackendError> {
        Err(BackendError::NotSupported)
    }

    async fn get_response_impl(
        &self,
        request: UnknownBlip,
    ) -> Result<Option<UnknownResponseItem>, BackendError> {
        Err(BackendError::NotSupported)
    }

    async fn get_responses_impl(
        &self,
        request: UnknownBlip,
    ) -> Result<Box<dyn Iterator<Item = UnknownResponseItem>>, BackendError> {
        Ok(Box::new(self.get_response_impl(request).await?.into_iter()))
    }
}
