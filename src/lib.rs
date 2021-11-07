use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

mod client;
mod database;
mod errors;
mod spider;

pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").ok();

    let api_cache = sled::Config::default()
        .path("data/api_cache")
        .use_compression(true)
        .open()
        .unwrap();

    let mut spider = crate::spider::Spider::new(google_cookie, api_cache);

    dbg!(
        spider
            .client
            .store_sku("8f006ae6f46648649e58b865b89b165ap")
            .await
    );

    // spider.crawl().await;
}
