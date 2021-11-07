use bounded_integer::BoundedU64;
use derive_more::Deref;
use eyre::{eyre, Result};
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct StoreSearchResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct StoreSkuResponse {
    game_id: String,
    sku_id: String,
    _2: Ignored,
    name: String,
    _4: Ignored,
    description: String,
    _6: Ignored,
    _7: Ignored,
    _8: Ignored,
    _9: Ignored,
    _10: Ignored,
    _11: Ignored,
    _12: Ignored,
    _13: Ignored,
    _14: Ignored,
    _15: Ignored,
    sku: Sku,
    _17: Ignored,
    _18: Ignored,
    _19: Ignored,
    _20: Ignored,
    _21: Ignored,
    _22: Ignored,
    _23: Ignored,
    #[serde(default)]
    _24: Ignored,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(transparent)]
pub struct Ignored(Option<Json>);

impl std::fmt::Debug for Ignored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_none() {
            write!(f, "null")
        } else {
            write!(f, "…")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct Sku {
    sku_id: String,
    name: String,
    _images: Ignored,
    _3: Ignored,
    game_id: String,
    internal_name: String,
    sku_type_id: u64,
    _7: Ignored,
    _8: Ignored,
    description: String,
    _10_timestamp: Ignored,
    _11: Ignored,
    _12: Ignored,
    _13: Ignored,
    _14: Ignored,
    publisher: Ignored,
    developers: Ignored,
    _17: Ignored,
    _18: Ignored,
    _19: Ignored,
    _20: Ignored,
    _21: Ignored,
    _22: Ignored,
    _23: Ignored,
    languages: Ignored,
    countries: Ignored,
    _26_timestamp: Ignored,
    _27: Ignored,
    _28: Ignored,
    _29: Ignored,
    _30: Ignored,
    _31: Ignored,
    _32: Ignored,
    _33: Ignored,
    _34: Ignored,
    _35: Ignored,
    _36: Ignored,
    #[serde(default)]
    _37: Ignored,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default, Deref)]
pub struct PlayerSearchResponse {
    #[serde(default)]
    _0: Ignored,
    #[deref]
    #[serde(default)]
    players: Vec<PlayerSearchResponsePlayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default, Deref)]
pub struct PlayerSearchResponsePlayer {
    #[deref]
    player: Player,
    #[serde(default)]
    _1: Ignored,
    #[serde(default)]
    _2: Ignored,
    #[serde(default)]
    _3: Ignored,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct Player {
    gamertag: PlayerGamertag,
    avatar: PlayerAvatar,
    _2: Ignored,
    gamertag_normalized: String,
    _4: Ignored,
    player_id: String,
    #[serde(default)]
    _6: Ignored,
    #[serde(default)]
    _7: Ignored,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct PlayerGamertag {
    name: String,
    number: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct PlayerAvatar {
    id: String,
    url: String,
}
