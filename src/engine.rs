use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use tokio::runtime::Handle;

use crate::never;
use crate::panic;
use crate::Blob;
use crate::Context;

#[derive(Debug)]
pub struct Engine<Storage: crate::Storage> {
    storage: Arc<Storage>,
    runtime: tokio::runtime::Handle,
}

impl<Storage: crate::Storage> Default for Engine<Storage>
where Storage: Default
{
    fn default() -> Self {
        Self {
            storage: Default::default(),
            runtime: Handle::current(),
        }
    }
}

impl<Storage: crate::Storage> Engine<Storage> {
    pub fn new(storage: Arc<Storage>) -> Engine<Storage> {
        Self {
            storage,
            runtime: Handle::current(),
        }
    }

    pub async fn execute<Request: crate::Request>(
        &self,
        request: Request,
    ) -> Result<Request::Response, never> {
        let request_blip = request.to_blob().blip();

        let context = Context::new(&request, &self.storage);

        let response = request.execute(&mut context).await?;

        self.storage.insert_response(&request, &response).await?;

        todo!()
    }
}
