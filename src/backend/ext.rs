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
