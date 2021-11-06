use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::client::Client;

pub struct Spider {
    pub client: Client,
}

impl Spider {
    pub fn new(cookie_header: Option<String>, api_cache: sled::Db) -> Self {
        Self {
            client: Client::new(cookie_header, api_cache),
        }
    }

    /// Seeds the spider with a few starting points from which to begin
    /// crawling.
    pub async fn seed(&mut self) {
        for first_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
            // self.client
            //     .store_search(&format!("{}", first_character))
            //     .await;
            for second_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                if let Err(err) = self
                    .client
                    .store_search(&format!("{}{}", first_character, second_character))
                    .await
                {
                    tracing::error!("{:#?}", err)
                }
            }
        }

        for first_character in "abcdefghijklmnopqrstuvwxyz".chars() {
            for second_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                if let Err(err) = self
                    .client
                    .player_search(&format!("{} {}", first_character, second_character))
                    .await
                {
                    tracing::error!("{:#?}", err)
                }
            }
        }
    }

    pub async fn crawl(&mut self) {
        tracing::info!("Spider is crawling");

        let result = self.client.store_search("the").await.unwrap();
        tracing::info!(
            "Searching... {}",
            result.to_string().chars().take(512).collect::<String>()
        );
    }
}
