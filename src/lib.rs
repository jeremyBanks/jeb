use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::client::ApiCacheBucket;

pub mod client;
pub mod errors;
pub mod spider;

#[tracing::instrument(name = "stadians")]
pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").expect("GOOGLE_COOKIE not set");

    let api_cache: ApiCacheBucket =
        kv::Store::new(kv::Config::new("cache.ignore").cache_capacity(536_870_912))
            .unwrap()
            .bucket(Some("v0"))
            .unwrap();

    let mut spider = crate::spider::Spider::new(google_cookie, api_cache);
    let result = spider
        .client
        .api_request(&[("FdyJ0", json!(["j e"]))])
        .await
        .unwrap();

    tracing::info!("we got something! {}", result[0].to_string());
}
