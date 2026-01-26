use crate::storage::Storage;

#[derive(Debug, Clone)]
struct WebStorage {
    pub base_url: String,
}

impl Storage for WebStorage {}
