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

        let seed_skus = vec![
            "053ebc72c9ff4de49e8ebf3b4ad0ce47p",
            "06adceee27be4150b080495b3c51c68c",
            "0b3acb4586bc4a049cac271ddc598e3e",
            "0b78bb41b4bc41df97a4762bb2829f39",
            "0db057f2ebf24cf0884d33d1676e588bp",
            "17affc92b9ca47a7b64e748609ed25af",
            "1e4107605f83447fa8e04e0abdc578b3",
            "1f50a7a214994a5cb861db76eef77564",
            "27099d8546ee4bcfbd172b5a4071c399p",
            "2a598d522e4f4f9d818d08fe582d17e3",
            "2e07db5d338d40cb9eac9deae4154f11",
            "2e51be1b06974b81bcf0b4767b4c63dfp",
            "2f112e5ba3d544d69bb1d537c5c4ae5c",
            "32a0791a88474f08ad7a687846b786f2",
            "35087d3c68c94b03b6f7e67440b8b283",
            "3e15e73f6b6541f69021289923ca8628p",
            "3e5dab36f21e466499f3449f13c98329",
            "41538c3158ec4c7aaf641ab7b9330d37p",
            "480a6d783c834b9faa5c995f69b49431",
            "53ed0e18ae494c929dbec8262afcc915",
            "54caf5d57b284ff28f060331f21b2eb7p",
            "57bdb73d1f6e47f99d1dfc61c1ca1402",
            "58f0a37bf6b141e4ba3a14b59e46b7ff",
            "5978ecf352094277a09daf94b5bfff1cp",
            "59c8314ac82a456ba61d08988b15b550",
            "5a2dadfd862645bc8f3c6d70e6529744p",
            "5c1d84fe250a473e9d0313ed232508bc",
            "5ce9f4c1253047dda226a982fc3dc866",
            "63e94c60b10e4267add34ef768c30f8f",
            "69f80c302be14b8284ba84d1229848e8",
            "6d54e2f977514da38090c19655c61badp",
            "6ed658c7e6564de6acf724f979172bb6p",
            "7265f354d9ff454d81c20428505e991f",
            "77ab8b1807a849e3991826d8d7a9e3e0",
            "7f69e006dede4c8eab019dede177c57dp",
            "843246253ef640a29bccebb19a404317",
            "85cc8be9f94b4a188cab0bdcafb7434a",
            "8e894e5800a84b8aa6b8e277e5790a53",
            "92d38339bd8a453fbfc124dcba5ed47bp",
            "a31dabb87e2441978da11fd60f3197dfp",
            "aba2b1bc01b14666b7fa48fcef18728dp",
            "bd70626ec3834dedbc6dda5b956f7648",
            "be9526126d394061b0eef9b16352357e",
            "c43eb98aaf8e455fb6faece785893268",
            "ca8344739a34441dac1f53938b55e178p",
            "cb723734e9204c51b81f01c3ae27947f",
            "d125c6e1d3514f479e34badaad5b9836",
            "d26a526c2e0e4aefb54fbf0f87783de0p",
            "d7ce926f62f3488999c0dc45d52fd946p",
            "da2272f0fb9a49e6aa6ba2a93b1e24dep",
            "decf585409804c8f9c1bd03562504e10",
            "dfcc2a3f9ab0421c86ba27e35ff8e41a",
            "ea65fc37d8504a518edad7534af3fc36",
            "ec30180d14de41fcaec12d442b81e248",
            "ee008ea2de714bdb811694c75a023d0f",
            "f00f5466907349c79467115296491b0fp",
            "f4934cfefbf845c3a3fa345dd65f9fcd",
            "f746502dc3b54a86ba5c42bb2e0ecea5",
            "f7aa7caf05e64d91af2063bf1803e947",
            "fea737af05af4408aaae16417011014f",
        ];

        for seed_sku in seed_skus {
            let r = self
                .client
                .api_request("FWhQV", &json!([null, seed_sku]))
                .await;
            println!("{:#?}", r);
        }

        return;

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
