#![feature(generic_const_exprs)]

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

    let record = json!("a value");

    let key = append_id(
        api_cache_key("foo", record),
        api_cache.generate_id().unwrap(),
    );

    println!("cache_key: {}", printable(&key, '_'));
}

fn printable(bytes: &[u8], filler: char) -> String {
    regex::Regex::new(r"[^\ -\~]")
        .unwrap()
        .replace_all(
            &String::from_utf8_lossy(&bytes).to_string(),
            filler.to_string(),
        )
        .to_string()
}

fn append_id<const T: usize>(array: [u8; T], id: u64) -> [u8; T + 8] {
    let mut result = [0u8; T + 8];
    result[..T].copy_from_slice(&array);
    result[T..].copy_from_slice(&id.to_be_bytes());
    result
}

fn api_cache_key(method_id: &str, parameters: Json) -> [u8; 128] {
    let mut key = [0u8; 128];

    let key_prefix = "api_cache_".as_bytes();
    key[0..10].copy_from_slice(key_prefix);

    let key_method_id: [u8; 10] = fit_into_array(method_id.as_bytes());
    key[10..20].copy_from_slice(&key_method_id);

    let parameters_json = parameters.to_string();
    let key_parameters: [u8; 108] = fit_into_array(parameters_json.as_bytes());
    key[20..128].copy_from_slice(&key_parameters);

    key
}

fn fit_into_array<const T: usize>(value: &[u8]) -> [u8; T] {
    let mut array = [0x00; T];

    if value.len() <= T {
        // If the value fits in the array, great, put it in.
        // If it's shorter than the array, we'll leave trailing zero-bytes.
        array[..value.len()].copy_from_slice(value);
    } else {
        // If the value's too large to fit in the array, hash it with blake3.
        let mut hasher = blake3::Hasher::new();
        hasher.update(value);
        let mut result = hasher.finalize_xof();

        // Fill the first half of the array with value, truncated to fit.
        array.copy_from_slice(&value[..T / 2]);
        // Fill the the second half from the hash digest.
        result.fill(&mut array[T / 2..]);
    }

    array
}
