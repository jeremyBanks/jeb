use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

mod client;
mod database;
mod errors;
mod spider;

pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").expect("GOOGLE_COOKIE not set");

    let api_cache = ();

    let mut spider = crate::spider::Spider::new(google_cookie, api_cache);

    spider.crawl().await
}
