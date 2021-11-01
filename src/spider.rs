use serde_json::json;

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

    pub async fn crawl(&mut self) {
        tracing::info!("crawling");

        for character_1 in "abcdefghijklmnopqrstuvwxyz".chars() {
            for character_2 in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                let result = self
                    .client
                    .api_request(
                        "FdyJ0",
                        &json!([format!("{} {}", character_1, character_2)]),
                    )
                    .await
                    .unwrap();
                tracing::info!("Searching for users... {}", result.to_string())
            }
        }
    }
}
