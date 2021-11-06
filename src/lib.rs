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

    let response = spider.client.player_search("j e").await.unwrap();
    let response = response.as_array().unwrap();
    let players = response.get(1).unwrap().as_array().unwrap();
    let players = players
        .iter()
        .map(|p| p.as_array().unwrap())
        .map(|p| PlayerResult {
            name: p
                .get(0)
                .unwrap()
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            number: p
                .get(0)
                .unwrap()
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_array()
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            id: p
                .get(0)
                .unwrap()
                .as_array()
                .unwrap()
                .get(5)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        })
        .collect::<Vec<_>>();

    println!("{:#?}", players);

    // spider.seed().await;
    // spider.crawl().await;
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerResult {
    name: String,
    number: String,
    id: String,
}
