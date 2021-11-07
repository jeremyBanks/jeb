use bounded_integer::BoundedU64;
use derive_more::Deref;
use eyre::{eyre, Result};
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Getters, Default)]
pub struct StoreSearchResponse {}

macro_rules! proto {
    (
        pub struct $name:ident;

        $(
            $(
                optional ::$optional_type:ident $optional:ident
            )*
            $(
                optional string $optional_string:ident
            )*
            $(
                optional uint64 $optional_uint64:ident
            )*
            $(
                repeated ::$repeated_type:ident $repeated:ident
            )*
            $(
                repeated string $repeated_string:ident
            )*
            $(
                repeated uint64 $repeated_uint64:ident
            )*
            $(
                reserved $ignored_prop:ident
            )*
            ;
        )+
    ) => {
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
            )+
        }

        // impl $name {
        //     pub fn new() -> Self {
        //         Self::default()
        //     }
        // }
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

proto! {
    pub struct StoreSkuResponse;
    optional string game_id;
    optional string sku_id;
    reserved _2;
    optional string name;
    reserved _4;
    optional string description;
    reserved _6;
    reserved _7;
    reserved _8;
    reserved _9;
    reserved _10;
    reserved _11;
    reserved _12;
    reserved _13;
    reserved _14;
    reserved _15;
    optional ::Sku sku;
    reserved _17;
    reserved _18;
    reserved _19;
    reserved _20;
    reserved _21;
    reserved _22;
    reserved _23;
    reserved _24;
}

proto! {
    pub struct Sku;
    optional string sku_id;
    optional string name;
    reserved _images;
    reserved _3;
    optional string game_id;
    optional string internal_name;
    optional uint64 sku_type_id;
    reserved _7;
    reserved _8;
    optional string description;
    reserved _10_timestamp;
    reserved _11;
    reserved _12;
    reserved _13;
    reserved _14;
    reserved publisher;
    reserved developers;
    reserved _17;
    reserved _18;
    reserved _19;
    reserved _20;
    reserved _21;
    reserved _22;
    reserved _23;
    reserved languages;
    reserved countries;
    reserved _26_timestamp;
    reserved _27;
    reserved _28;
    reserved _29;
    reserved _30;
    reserved _31;
    reserved _32;
    reserved _33;
    reserved _34;
    reserved _35;
    reserved _36;
    reserved _37;
}

proto! {
    pub struct PlayerSearchResponse;
    reserved _0;
    repeated ::PlayerSearchResponsePlayer players;
}

proto! {
    pub struct PlayerSearchResponsePlayer;
    optional ::Player player;
    reserved _1;
    reserved _2;
    reserved _3;
}

proto! {
    pub struct Player;
    optional ::PlayerGamertag gamertag;
    optional ::PlayerAvatar avatar;
    reserved _2;
    optional string gamertag_normalized;
    reserved _4;
    optional string player_id;
    reserved _6;
    reserved _7;
}

proto! {
    pub struct PlayerGamertag;
    optional string name;
    optional string number;
}

proto! {
    pub struct PlayerAvatar;
    optional string id;
    optional string url;
}
