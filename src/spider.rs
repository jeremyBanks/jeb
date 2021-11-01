use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    MyCaptures { page: u64 },
    MyGames,
    MyPlayer,
    PlayerFriends { player_id: u64 },
    PlayerGameStats { player_id: u64, game_id: String },
    PlayerGames { player_id: u64 },
    PlayerProfile { player_id: u64 },
    PlayerSearch { name_prefix: String },
    StoreGame { game_id: String },
    StoreGameSku { game_id: String, sku_id: String },
    StoreList { list_id: String },
}

impl ApiRecordId {
    pub fn execute(&self, client: &mut Client) -> eyre::Result<Json> {
        match self {
            Self::MyCaptures { page } => client.my_captures(page),
            Self::MyGames => client.my_games(),
            Self::MyPlayer => client.my_player(),
            Self::PlayerFriends { player_id } => client.player_friends(player_id),
            Self::PlayerGameStats { player_id, game_id } => client.player_game_stats(player_id, game_id),
            Self::PlayerGames { player_id } => client.player_games(player_id),
            Self::PlayerProfile { player_id } => client.player_profile(player_id),
            Self::PlayerSearch { name_prefix } => client.player_search(name_prefix),
            Self::StoreGame { game_id } => client.store_game(game_id),
            Self::StoreGameSku { game_id, sku_id } => client.store_game_sku(game_id, sku_id),
            Self::StoreList { list_id } => client.store_list(list_id),
        }

    }
}

/// Record corresponding to a logical model, typically derived from one or more API records.
#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Clone)]
enum ModelRecordId {
    Game { game_id: String },
    Sku { sku_id: String },
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

        for list_id in [
            // all game and game bundle skus currently available (not delisted or preorders).
            3,    // preorder game and game bundle skus.
            76,   // stadia pro skus.
            2001, // ubisoft+ skus.
            2002,
        ] {
            seed(ApiRecordId::StoreList {
                list_id: format!("{}", list_id),
            });
        }

        for character_1 in "abcdefghijklmnopqrstuvwxyz".chars() {
            for character_2 in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                seed(ApiRecordId::PlayerSearch {
                    name_prefix: format!("{}{}", character_1, character_2),
                });
            }
        }
    }

    pub async fn crawl(&mut self) {
        tracing::info!("Spider is crawling");

        let result = self.client.player_search("j eremy").await.unwrap();
        tracing::info!("Searching for users... {}", result.to_string());
    }
}
