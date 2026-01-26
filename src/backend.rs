mod error;
pub mod ext;
pub mod impl_;
mod lazy;

use async_trait::async_trait;

pub use self::ext::BackendExt;
pub use self::impl_::BackendImpl;

pub type Backend = Arc<dyn BackendImpl>;

pub static USER_LOCAL: Lazy<Backend> = Lazy::new(|| {
    info!("Initializing backend with user-local disk storage.");
    let mut path = home::home_dir().unwrap_or_default();
    path.push(format!(".{}", env!("CARGO_CRATE_NAME", "fiction")));
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path.push("db");
    let backend = SqliteStorage::open(path.to_string_lossy().as_ref()).unwrap();
    Arc::new(backend)
});

pub struct ResponseItem<Request: crate::Request> {
    pub request: Blip<Request>,
    pub response: Blip<Request::Response>,
    pub metadata: Blip<Metadata>,
}

// XXX: why are these not even taking self
// oh dear

#[async_trait]
pub trait BackendExt {
    /// Stores this [`Blob`] if it's too long to inline, and returns the
    /// respective [`Blip`].
    async fn insert_blob<T: Blobbable + ?Sized>(blob: Blob<T>) -> Result<Blip<T>, BackendError> {
        Err(BackendError::NotSupported.into())
    }

    /// Retrieves the [`Blob`] corresponding to the given [`Blip`], if it's
    /// stored.
    async fn get_blob<T: Blobbable + ?Sized>(
        blip: Blip<T>,
    ) -> Result<Option<Blob<T>>, BackendError> {
        Err(BackendError::NotSupported.into())
    }

    /// Get a [`Response`] for the given [`Request`], either from storage or by
    /// executing it.
    async fn get<Request: crate::Request>(
        &self,
        request: &Request,
    ) -> Result<Request::Response, BackendError> {
        Err(BackendError::NotSupported.into())
    }

    /// Get a [`Response`] for the given [`Request`], either from storage or by
    /// executing it.
    async fn set<Request: crate::Request>(
        &self,
        request: &Request,
        response: &Request::Response,
    ) -> Result<(), BackendError> {
        Err(BackendError::NotSupported.into())
    }

    /// Get a [`Blip`] of [`Response`] for the given [`Request`] [`Blip`],
    /// either from storage or by executing it if possible.
    async fn get_blip<Request: crate::Request>(
        &self,
        request: Blip<Request>,
    ) -> Result<Blip<Request::Response>, BackendError> {
        Err(BackendError::NotSupported.into())
    }

    /// Sets the [`Response`] [`Blip`] for the given [`Request`] [`Blip`].
    async fn set_blip<Request: crate::Request>(
        &self,
        request: Blip<Request>,
        response: Blip<Request>,
    ) -> Result<(), BackendError> {
        Err(BackendError::NotSupported.into())
    }
}

/// Untyped Response item used in [`ResponseImpl`].
pub struct UnknownResponseItem {
    pub request: UnknownBlip,
    pub response: UnknownBlip,
    pub metadata: Blip<Metadata>,
}
