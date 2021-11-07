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
                optional $prop:ident [type = $type:ty]
            )*
            $(
                repeated $repeated_prop:ident [type = $repeated_type:ty]
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
                    pub $prop: $type,
                )*
                $(
                    #[serde(default)]
                    pub $repeated_prop: Vec<$repeated_type>,
                )*
                $(
                    #[serde(default)]
                    $ignored_prop: Ignored,
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
    optional game_id [type = String];
    optional sku_id [type = String];
    reserved _2;
    optional name [type = String];
    reserved _4;
    optional description [type = String];
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
    optional sku [type = Sku];
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
    optional sku_id [type = String];
    optional name [type = String];
    reserved _images;
    reserved _3;
    optional game_id [type = String];
    optional internal_name [type = String];
    optional sku_type_id [type = u64];
    reserved _7;
    reserved _8;
    optional description [type = String];
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
    optional players [type = Vec<PlayerSearchResponsePlayer>];
}

proto! {
    pub struct PlayerSearchResponsePlayer;
    optional player [type = Player];
    reserved _1;
    reserved _2;
    reserved _3;
}

proto! {
    pub struct Player;
    optional gamertag [type = PlayerGamertag];
    optional avatar [type = PlayerAvatar];
    reserved _2;
    optional gamertag_normalized [type = String];
    reserved _4;
    optional player_id [type = String];
    reserved _6;
    reserved _7;
}

proto! {
    pub struct PlayerGamertag;
    optional name [type = String];
    optional number [type = String];
}

proto! {
    pub struct PlayerAvatar;
    optional id [type = String];
    optional url [type = String];
}
