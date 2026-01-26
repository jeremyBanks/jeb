use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

// use crate::backend::Backend;
// use crate::backend::BackendError;
// use crate::backend::BackendImpl;
use crate::blobs::UnknownBlip;
use crate::never;
use crate::AnyRequest;
use crate::AnyResponse;
use crate::Blip;
use crate::Blob;
use crate::PhantomType;
use crate::Request;
use crate::Response;

/// A context is associated with a [`Request`] instance and manages all of its
/// interactions with the rest of the [`Engine`]. If the request produces a new
/// [`Response`], the [`Context`] is consumed to produce its [`Metadata`].
#[derive(Debug)]
pub struct Context {
    pub request: Blip<AnyRequest>,
    pub backend: Backend,
    pub aliases: Vec<Blip<AnyRequest>>,
}

/// Metadata associated with the production of a given [`Response`].
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub validated_at: i64,
    pub created_at: i64,
    pub read: Vec<Blob<AnyRequest>>,
    pub written: Vec<Blob<AnyRequest>>,
}

impl Context {
    /// Adds an alias request that will also be associated with this request's
    /// result.
    pub fn alias(&self, request: AnyRequest) {}
}

// #[async_trait]
// impl BackendImpl for Context {}
