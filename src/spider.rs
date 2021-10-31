use crate::client::{ApiCacheBucket, Client};

pub struct Spider {
    pub client: Client,
}

impl Spider {
    pub fn new(cookie_header: String, api_cache: ApiCacheBucket) -> Self {
        Self {
            client: Client::new(cookie_header, api_cache),
        }
    }
}
