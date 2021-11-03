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

    let record = json!("a9io8uh3r ih329hf923hfil23hggitl2h3iolugb23i7gh23ilugbi2y3kbgi2lu3i7g23bgiu23bgti723biou32bi27wb2iou3gbi73vfj,sb3u7g3awiugb34swiugsk3gb3skjgb3skgb3kwbgiku3bklg3biub3iugbiu3bgo.3wsgi3bskug3bsk, value");

    let key = append_id(
        api_cache_key("foo", record),
        api_cache.generate_id().unwrap(),
    );

    println!("cache_key: {}", printable(&key, '_'));
}

fn printable(bytes: &[u8], filler: char) -> String {
    regex::Regex::new(r"[^ -~]")
        .unwrap()
        .replace_all(
            &String::from_utf8_lossy(bytes).to_string(),
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

fn api_cache_key(method_id: &str, parameters: Json, status: CallStatus) -> [u8; 128] {
    let mut key = [0u8; 128];

    let key_prefix = "api_cache_".as_bytes();
    key[0..10].copy_from_slice(key_prefix);

    let key_method_id: [u8; 10] = fit_into_array(method_id.as_bytes());
    key[10..20].copy_from_slice(&key_method_id);

    let parameters_json = parameters.to_string();
    let key_parameters: [u8; 107] = fit_into_array(parameters_json.as_bytes());
    key[20..127].copy_from_slice(&key_parameters);

    key[127] = status as u8;

    key
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum CallStatus {
    /// This call has been seeded into the database, but not executed.
    Known = 0x00,
    /// This call has been attempted, but we don't know the result.
    Attempted = 0x10,
    /// This call failed for out-of-band reasons (i.e. network error, unexpected
    /// response format).
    Unable = 0x20,
    /// The call failed with an in-band error response value.
    Error = 0x30,
    /// The call succeeded with a successful but empty response value.
    Empty = 0x35,
    /// The call succeeded with a successful non-empty response value.
    Full = 0x40,
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
        array[..T / 2].copy_from_slice(&value[..T / 2]);
        // Fill the the second half from the hash digest.
        result.fill(&mut array[T / 2..]);
    }

    array
}
