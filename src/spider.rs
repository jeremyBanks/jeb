use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::client::{ApiCacheBucket, Client};

pub struct Spider {
    pub client: Client,
}

#[derive(Debug, From, Serialize, Deserialize, Clone)]
enum RecordId {
    ApiRecordId(ApiRecordId),
    ModelRecordId(ModelRecordId),
}

/// Record corresponding to a raw API call.
#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Clone)]
enum ApiRecordId {
    StoreGame { game_id: String },
    StoreGameSku { game_id: String, sku_id: String },
    StoreSearch { name_contains: String },
}

impl ApiRecordId {
    pub fn execute(&self, client: &mut Client) -> eyre::Result<Json> {
        unimplemented!()
    }
}

/// Record corresponding to a logical model, typically derived from one or more API records.
#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Clone)]
enum ModelRecordId {
    Game { game_id: String },
    Sku { sku_id: String },
}

impl ModelRecordId {
    pub fn requires(&self) -> Vec<ApiRecordId> {
        unimplemented!()
    }
}

impl Spider {
    pub fn new(cookie_header: String, api_cache: ApiCacheBucket) -> Self {
        Self {
            client: Client::new(cookie_header, api_cache),
        }
    }

    /// Seeds the spider with a few starting points from which to begin crawling.
    pub async fn seed(&mut self) {
        fn seed(_id: impl Into<RecordId>) {}

        for first_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
            for second_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                seed(ApiRecordId::StoreSearch {
                    name_contains: format!("{}{}", first_character, second_character),
                });
            }
        }
    }

    pub async fn crawl(&mut self) {
        tracing::info!("Spider is crawling");

        let result = self.client.store_search("e").await.unwrap();
        tracing::info!(
            "Searching... {}",
            result.to_string().chars().take(512).collect::<String>()
        );
    }
}
