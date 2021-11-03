use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

mod client;
mod database;
mod errors;
mod spider;

pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").expect("GOOGLE_COOKIE not set");

    let mut api_cache = sled::open("data/api_cache").unwrap();

    let mut spider = crate::spider::Spider::new(google_cookie, ());

    spider.crawl().await
}

fn api_cache_key(method_id: &str, parameters: Vec<Json>) -> [u8; 256] {
    let mut key = [b'_'; 256];
    let rest = &mut key;

    let (key_prefix, rest) = rest.split_at_mut(16);
    let prefix = "api_cache";
    key_prefix[..prefix.len()].copy_from_slice(prefix.as_bytes());

    let (key_method_id, rest) = rest.split_at_mut(16);
    key_method_id[..method_id.len()].copy_from_slice(method_id.as_bytes());

    let (key_parameters, rest) = rest.split_at_mut(224);
    let parameters_json = json!(parameters).to_string();
    key_parameters[..parameters_json.len()].copy_from_slice(parameters_json.as_bytes());

    assert!(rest.is_empty());
    key
}
