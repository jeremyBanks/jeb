use serde_json::{json, Value as Json};

pub mod client;
pub mod errors;
pub mod spider;

#[tracing::instrument(name = "stadians")]
pub async fn main() {
    let google_cookie = std::env::var("GOOGLE_COOKIE").expect("GOOGLE_COOKIE not set");
    let db = rusqlite::Connection::open("./x.sqlite").expect("unable to open sqlite db");

    let mut spider = crate::spider::Spider::new(google_cookie);
    let result = spider
        .client
        .api_request(&[("D0Amud", json!([Json::Null, true]))])
        .await
        .unwrap();
    tracing::info!("we got something! {}", result[0].to_string());
}
