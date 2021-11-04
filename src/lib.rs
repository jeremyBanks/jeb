use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

mod client;
mod database;
mod errors;
mod spider;

pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").expect("GOOGLE_COOKIE not set");

    let api_cache = sled::open("data/api_cache").unwrap();

    let mut spider = crate::spider::Spider::new(google_cookie, api_cache.clone());

    spider.seed().await;
    spider.crawl().await;
}
