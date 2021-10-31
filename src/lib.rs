pub mod client;
pub mod errors;
pub mod spider;

#[tracing::instrument(name = "stadians")]
pub async fn main() {
    // let credentials = credentials::GoogleCookies::try_from_env()
    //     .expect("Expected Google credentials in environment variables");

    // let mut spider = crate::spider::Spider::new(credentials);
    // let result = spider.client.http_post("settings").await.unwrap();
    // let length = result.len();
    // tracing::info!(length, "we got some bytes!");
}
