use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::client::ApiCacheBucket;

pub mod client;
pub mod errors;
pub mod spider;

pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").expect("GOOGLE_COOKIE not set");

    let api_cache: ApiCacheBucket = kv::Store::new(kv::Config::new("cache.ignore"))
        .unwrap()
        .bucket(Some("v0"))
        .unwrap();

    let mut spider = crate::spider::Spider::new(google_cookie, api_cache);

    spider.crawl().await
}
