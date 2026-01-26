use async_trait::async_trait;

use crate::backend::BackendError;
use crate::backend::BackendImpl;
use crate::backend::UnknownResponseItem;
use crate::blobs::bytes;
use crate::blobs::UnknownBlip;
use crate::blobs::UnknownBlob;
use crate::Blip;
use crate::Blob;
use crate::Metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// No-op dummy storage backend. Retains nothing, without error.
pub struct NoStorage;

#[async_trait]
impl BackendImpl for () {}

#[async_trait]
impl BackendImpl for NoStorage {
    async fn insert_blob_impl(&self, blob: &UnknownBlob) -> Result<UnknownBlip, BackendError> {
        Ok(blob.blip())
    }

    async fn get_blob_impl(&self, blip: UnknownBlip) -> Result<Option<UnknownBlob>, BackendError> {
        Ok(None)
    }

    async fn insert_response_impl(
        &self,
        request: UnknownBlip,
        response: UnknownBlip,
        metadata: UnknownBlip,
    ) -> Result<UnknownResponseItem, BackendError> {
        Ok(UnknownResponseItem {
            request,
            response,
            metadata,
        })
    }

    async fn get_response_impl(
        &self,
        request: UnknownBlip,
    ) -> Result<Option<UnknownResponseItem>, BackendError> {
        Ok(None)
    }
}
