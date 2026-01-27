use bounded_integer::BoundedU64;
use derive_more::Deref;
use eyre::{eyre, Result};
use getset::Getters;
use paste::paste;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct StoreSearchResponse {}

macro_rules! protos {
    ($(
        pub struct $name:ident {
            $(
                $(
                    optional self::$optional_type:ident $optional:ident = $_id1:tt
                )*
                $(
                    optional string $optional_string:ident = $_id2:tt
                )*
                $(
                    optional uint64 $optional_uint64:ident = $_id3:tt
                )*
                $(
                    repeated self::$repeated_type:ident $repeated:ident = $_id4:tt
                )*
                $(
                    repeated string $repeated_string:ident = $_id5:tt
                )*
                $(
                    repeated uint64 $repeated_uint64:ident = $_id6:tt
                )*
                $(
                    reserved $($reserved:tt),+
                )*
                ;
            )+
        }
    )+) => {
        paste!{$(
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
        pub struct $name {
            $(
                $(
                    #[serde(default)]
                    pub $optional: $optional_type,
                )*
                $(
                    #[serde(default)]
                    pub $optional_string: String,
                )*
                $(
                    #[serde(default)]
                    pub $optional_uint64: u64,
                )*
                $(
                    #[serde(default)]
                    pub $repeated: Vec<$repeated_type>,
                )*
                $(
                    #[serde(default)]
                    pub $repeated_string: Vec<String>,
                )*
                $(
                    #[serde(default)]
                    pub $repeated_uint64: Vec<u64>,
                )*
                $($(
                    #[serde(default, skip_serializing)]
                    [<_ $reserved>]: Ignored,
                )+)*
            )+
        })+
        }
    };
}

impl std::fmt::Debug for Ignored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_none() {
            write!(f, "null")
        } else {
            write!(f, "…")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(transparent)]
pub struct Ignored(Option<Json>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u64)]
pub enum SkuType {
    Unknown = 0,

    Game = 1,
    Addon = 2,
    Bundle = 3,
    ExternalSubscription = 4,
    StadiaSubscription = 5,
    AddonSubscription = 6,
    AddonBundle = 9,
    PreorderBundle = 10,
}

impl Default for SkuType {
    fn default() -> Self {
        SkuType::Unknown
    }
}

protos! {
    pub struct StoreSkuResponse {
        optional string game_id = 1;
        optional string sku_id = 2;
        reserved 3;
        optional string name = 4;
        reserved 5;
        optional string description = 6;
        reserved 7, 8, 9, 10, 11, 12, 13, 14, 15, 16;
        optional self::Sku sku = 17;
        reserved 18, 19, 20, 21, 22, 23, 24, 25;
    }

    pub struct Sku {
        optional string sku_id = 1;
        optional string name = 2;
        reserved 3, 4;
        optional string game_id = 5;
        optional string internal_name = 6;
        optional self::SkuType sku_type = 7;
        reserved 8, 9;
        optional string description = 10;
        reserved 11, 12, 13, 14, 15, 16, 17, 18, 19, 20;
        reserved 21, 22, 23, 24, 25, 26, 27, 28, 29, 30;
        reserved 31, 32, 33, 34, 35, 36, 37, 38;
    }

    pub struct PlayerSearchResponse {
        reserved 1;
        repeated self::PlayerSearchResponsePlayer players = 2;
    }

    pub struct PlayerSearchResponsePlayer {
        optional self::Player player = 1;
        reserved 2, 3, 4;
    }

    pub struct Player {
        optional self::PlayerGamertag gamertag = 1;
        optional self::PlayerAvatar avatar = 2;
        reserved 3;
        optional string gamertag_normalized = 4;
        reserved 5;
        optional string player_id = 6;
        reserved 7, 8;
    }

    pub struct PlayerGamertag {
        optional string name = 1;
        optional string number = 2;
    }

    pub struct PlayerAvatar {
        optional string id = 1;
        optional string url = 2;
    }
}
