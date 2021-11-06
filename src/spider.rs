use std::{collections::HashSet, fmt::Write};

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

    pub async fn crawl(&mut self) {
        tracing::info!("Spider is crawling");

        let mut all_skus = HashSet::<crate::client::Sku>::new();

        for first_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
            for second_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                let prefix = format!("{}{}", first_character, second_character);
                let skus = self.client.store_search(&prefix).await;
                let skus = match skus {
                    Ok(skus) => skus,
                    Err(err) => {
                        tracing::error!("{}", err);
                        continue;
                    }
                };
                tracing::debug!(
                    "Found {:?} skus containing {:?}. {:#?}",
                    skus.len(),
                    &prefix,
                    skus.get(0),
                );
                all_skus.extend(skus);
            }
        }

        tracing::info!("Found {} skus in total.", all_skus.len());

        {
            let mut lines = String::new();
            let mut all_skus: Vec<_> = all_skus.into_iter().collect();
            all_skus.sort();
            for sku in all_skus {
                writeln!(lines, "{:36}/{:33} # {}", sku.game_id, sku.sku_id, sku.name).unwrap();
            }
            std::fs::write("data/skus.txt", lines).unwrap();
        }

        let mut prefixes = Vec::<String>::new();

        for first_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
            for second_character in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                prefixes.push(format!("{}{}", first_character, second_character));
            }
        }

        while !prefixes.is_empty() {
            let prefix = prefixes.pop().unwrap();
            let players = self.client.player_search(&prefix).await;
            let players = match players {
                Ok(players) => players,
                Err(err) => {
                    tracing::error!("{}", err);
                    continue;
                }
            };

            tracing::debug!(
                "Found {:?} players starting with {:?}. {:#?}",
                players.len(),
                &prefix,
                players.get(0),
            );
            if players.len() == 100 {
                if prefix.len() == 15 {
                    panic!("too many {:?}", prefix);
                }
                for additional in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                    prefixes.push(format!("{}{}", prefix, additional));
                }
            }
        }
    }
}
