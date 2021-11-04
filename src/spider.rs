use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::client::Client;

pub struct Spider {
    pub client: Client,
}

impl Spider {
    pub fn new(cookie_header: String, api_cache: sled::Db) -> Self {
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
                self.client
                    .store_search(&format!("{}{}", first_character, second_character))
                    .await;
            }
        }

        for first_character in "abcdefghijklmnopqrstuvwxyz".chars() {
            for second_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                self.client
                    .player_search(&format!("{} {}", first_character, second_character))
                    .await;
            }
        }
    }

    pub async fn crawl(&mut self) {
        tracing::info!("Spider is crawling");

        // L6k8hf[true] to explore ?
        // DalZif[null,[[\"27:6:CmYKZENoSUlDQklPQ2d3STlkSDlpd1lRMFBhVGh3RUtFZ2dCRWc0S0RBaUtsdmVLQmhDZ3BQV05BUW9SQ0FjU0RRb0xDSUtVMjRnR0VJaWtxaHdRQVJvTUNQNzhrSXdHRU1EbXRJQUM=\",\"CBsyaApmCmRDaElJQ0JJT0Nnd0k5ZEg5aXdZUTBQYVRod0VLRWdnQkVnNEtEQWlLbHZlS0JoQ2dwUFdOQVFvUkNBY1NEUW9MQ0lLVTI0Z0dFSWlrcWh3UUFSb01DUDc4a0l3R0VNRG10SUFD\"]]]",null,"1"]]]
        // to pagination exploration?
        // VS291[[null,"5325f670b13d4c959123c437948d1834rcp1\"]] to follow a game
        // kLgZB[[null,"..."]] to unfollow a game

        let result = self.client.store_search("the").await.unwrap();
        tracing::info!(
            "Searching... {}",
            result.to_string().chars().take(512).collect::<String>()
        );
    }
}
