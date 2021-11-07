use bounded_integer::BoundedU64;
use derive_more::Deref;
use eyre::{eyre, Result};
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
#[serde(default)]
pub struct StoreSearchResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
#[serde(default)]
pub struct StoreSkuResponse {
    game_id: String,
    sku_id: String,
    _2: Json,
    name: String,
    _4: Json,
    description: String,
    _6: Json,
    _7: Json,
    _8: Json,
    _9: Json,
    _10: Json,
    _11: Json,
    _12: Json,
    _13: Json,
    _14: Json,
    _15: Json,
    sku: Sku,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
#[serde(default)]
pub struct Sku {
    sku_id: String,
    name: String,
    _images: Json,
    _3: Json,
    game_id: String,
    internal_name: String,
    sku_type_id: u64,
    _7: Json,
    _8: Json,
    description: String,
    _10_timestamp: Json,
    _11: Json,
    _12: Json,
    _13: Json,
    _14: Json,
    publisher: String,
    developers: Vec<String>,
    _17: Json,
    _18: Json,
    _19: Json,
    _20: Json,
    _21: Json,
    _22: Json,
    _23: Json,
    languages: Vec<String>,
    countries: Vec<String>,
    _26_timestamp: Json,
    _27: Json,
    _28: Json,
    _29: Json,
    _30: Json,
    _31: Json,
    _32: Json,
    _33: Json,
    _34: Json,
    _35: Json,
    _36: Json,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default, Deref)]
#[serde(default)]
pub struct PlayerSearchResponse {
    _0: Json,
    #[deref]
    players: Vec<PlayerSearchResponsePlayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default, Deref)]
#[serde(default)]
pub struct PlayerSearchResponsePlayer {
    #[deref]
    player: Player,
    _1: Json,
    _2: Json,
    _3: Json,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
#[serde(default)]
pub struct Player {
    gamertag: PlayerGamertag,
    avatar: PlayerAvatar,
    _2: Json,
    gamertag_normalized: String,
    _4: Json,
    player_id: String,
    _6: Json,
    _7: Json,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
#[serde(default)]
pub struct PlayerGamertag {
    name: String,
    number: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
#[serde(default)]
pub struct PlayerAvatar {
    id: String,
    url: String,
}
